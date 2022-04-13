mod filter;
mod ifunny;

use crate::bot::commands::image::filter::Filter;
use crate::bot::commands::image::ifunny::add_ifunny_watermark;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::{ColorType, ImageEncoder, ImageFormat};
use photon_rs::channels::invert;
use photon_rs::colour_spaces::hue_rotate_hsv;
use photon_rs::conv::{gaussian_blur, sharpen};
use photon_rs::effects::{adjust_contrast, colorize, frosted_glass, inc_brightness, solarize};
use photon_rs::monochrome::grayscale_human_corrected;
use photon_rs::noise::add_noise_rand;
use photon_rs::transform::{fliph, flipv, resize, SamplingFilter};
use photon_rs::PhotonImage;
use reqwest::{get, Url};
use serenity::client::Context as SContext;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use std::collections::VecDeque;
use std::io::Cursor;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

// 500 MiB
const MAX_IMAGE_SIZE: u64 = 500 * 1024 * 1024;

#[derive(Debug, Copy, Clone)]
enum Transformation {
    Invert,
    Greyscale,
    Fliph,
    Flipv,
    Noise,
    Frost,
    Solarise,
    Colourise,
    Ifunny,
    Blur(i32),
    Contrast(f32),
    Huerotate(f32),
    Brighten(u8),
    Jpeg(u8),
    Resize((f32, f32)),
    Sharpen(u8),
    Filter(Filter),
}

impl FromStr for Transformation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Transformation::*;
        match s.to_lowercase().as_ref() {
            "invert" => Ok(Invert),
            "greyscale" | "grayscale" => Ok(Greyscale),
            "fliph" | "flipx" => Ok(Fliph),
            "flipv" | "flipy" => Ok(Flipv),
            "noise" => Ok(Noise),
            "frost" => Ok(Frost),
            "solarise" | "solarize" => Ok(Solarise),
            "colourise" | "colorize" => Ok(Colourise),
            "ifunny" => Ok(Ifunny),
            s => {
                let (t, amount) = s
                    .split_once('=')
                    .with_context(|| anyhow!("{} did not match any of the parameterless verbs and did not contain an `=`", s))?;

                match t {
                    "blur" => Ok(Blur(amount.parse()?)),
                    "contrast" => Ok(Contrast(amount.parse()?)),
                    "huerotate" => Ok(Huerotate(amount.parse()?)),
                    "brighten" => Ok(Brighten(amount.parse()?)),
                    "jpeg" => Ok(Jpeg(amount.parse()?)),
                    "resize" => Ok(Resize(f32ratio_amount(amount)?)),
                    "sharpen" => Ok(Sharpen(amount.parse()?)),
                    "filter" => Ok(Filter(amount.parse()?)),
                    _ => Err(anyhow!("Unknown transformation")),
                }
            }
        }
    }
}

impl Transformation {
    pub(crate) fn apply(self, mut image: PhotonImage) -> Result<PhotonImage> {
        use Transformation::*;
        match self {
            Invert => invert(&mut image),
            Fliph => fliph(&mut image),
            Flipv => flipv(&mut image),
            Noise => image = add_noise_rand(image),
            Greyscale => grayscale_human_corrected(&mut image),
            Frost => frosted_glass(&mut image),
            Solarise => solarize(&mut image),
            Colourise => colorize(&mut image),
            Ifunny => image = add_ifunny_watermark(image),
            Blur(radius) => gaussian_blur(&mut image, radius),
            Contrast(c) => adjust_contrast(&mut image, c),
            Huerotate(d) => hue_rotate_hsv(&mut image, d),
            Brighten(value) => inc_brightness(&mut image, value),
            Jpeg(q) => image = jpeg_encode(image, q)?,
            Resize((a, b)) => {
                let width = (image.get_width() as f32 * a) as u32;
                let height = (image.get_height() as f32 * b) as u32;
                if width == 0 || height == 0 {
                    return Err(anyhow!("Resize to 0 width or height"));
                } else if (width * height * 4) as u64 > MAX_IMAGE_SIZE {
                    return Err(anyhow!("Resize too large"));
                } else {
                    image = resize(&image, width, height, SamplingFilter::CatmullRom);
                }
            }
            Sharpen(n) => (0..n).for_each(|_| sharpen(&mut image)),
            Filter(f) => f.apply(&mut image),
        }
        Ok(image)
    }
}

#[derive(Debug, Parser)]
struct TransformationOpt {
    #[clap(short, conflicts_with = "image")]
    user: Option<u64>,
    #[clap(short, conflicts_with = "user")]
    image: Option<String>,
    transformations: Vec<Transformation>,
}

impl TransformationOpt {
    pub fn apply_all_transformations(&self, image: PhotonImage) -> Result<PhotonImage> {
        self.transformations
            .iter()
            .fold(Ok(image), |r, t| r.and_then(|i| t.apply(i)))
    }
}

#[group]
#[commands(transform, getpfp)]
pub(crate) struct Image;

#[command]
async fn transform(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let mut to_parse = args.iter::<String>().collect::<Result<VecDeque<_>, _>>()?;
    to_parse.push_front("transform".to_string());
    let opt: TransformationOpt = TransformationOpt::try_parse_from(&to_parse)?;

    let mut url = if let Some(user_id) = opt.user {
        let guild = msg
            .guild(ctx)
            .await
            .context("Message not sent in a guild")?;
        let user = guild
            .member(ctx, user_id)
            .await
            .context("Could not find member in guild")?
            .user;
        user.face()
    } else {
        opt.image.clone().unwrap_or_else(|| msg.author.face())
    };
    if url.starts_with('<') && url.ends_with('>') {
        let mut chars = url.chars();
        chars.next();
        chars.next_back();
        url = chars.collect();
    }
    let mut image = download_image(url).await?;

    image = timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || opt.apply_all_transformations(image)),
    )
    .await
    .context("Processing timed out")?
    .context("Failed to join thread")?
    .context("Could not process image")?;

    respond_with_image(ctx, msg, &msg.author.name, image).await?;
    Ok(())
}

#[command]
async fn getpfp(ctx: &SContext, msg: &Message) -> CommandResult {
    let u = msg.author.face();
    msg.reply(ctx, u).await?;
    Ok(())
}

fn get_format(url: &str) -> Result<ImageFormat> {
    let mut url = Url::parse(url).unwrap();
    url.set_query(None);
    let url = url.to_string();
    ImageFormat::from_path(url).context("Unknown image format")
}

async fn download_image(url: String) -> Result<PhotonImage> {
    let format =
        get_format(&url).context("Could not determine format when attempting to download image")?;

    let response = get(&url)
        .await
        .with_context(|| anyhow!("Failed to get response from {}", &url))?;

    if let Some(len) = response.content_length() {
        if len > MAX_IMAGE_SIZE {
            return Err(anyhow!("Image is too large"));
        }
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to get bytes from GET response")?;

    let image = if format == ImageFormat::WebP {
        webp::Decoder::new(&bytes)
            .decode()
            .context("Failed to load WebP image")?
            .to_image()
    } else {
        image::load_from_memory_with_format(&bytes, format)
            .with_context(|| anyhow!("Could not load image with format {:?}", format))?
    };

    let raw_pixels = image.to_rgba8().to_vec();
    Ok(PhotonImage::new(raw_pixels, image.width(), image.height()))
}

async fn respond_with_image(
    ctx: &SContext,
    msg: &Message,
    filename: &str,
    image: PhotonImage,
) -> Result<Message> {
    // Always send the image in PNG format
    let image = png_encode(image).context("Failed to encode image for reply")?;
    let pixels = image.get_raw_pixels();
    let filename = format!("{}.png", filename);
    let files = vec![(pixels.as_slice(), filename.as_str())];
    msg.channel_id
        .send_files(ctx, files.into_iter(), |m| {
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_users())
        })
        .await
        .context("Failed to send message")
}

fn png_encode(image: PhotonImage) -> Result<PhotonImage> {
    let mut cursor = Cursor::new(Vec::new());
    let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
    let pixels = image.get_raw_pixels();
    encoder
        .write_image(
            pixels.as_slice(),
            image.get_width(),
            image.get_height(),
            ColorType::Rgba8,
        )
        .context("Failed to write image using encoder")?;
    cursor.set_position(0);
    let image = PhotonImage::new(cursor.into_inner(), image.get_width(), image.get_height());
    Ok(image)
}

fn jpeg_encode(image: PhotonImage, quality: u8) -> Result<PhotonImage> {
    // Create a new cursor to write to and then read from.
    let mut cursor = Cursor::new(Vec::new());
    // Create a new encoder and write the image to the cursor.
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
    encoder
        .write_image(
            &image.get_raw_pixels(),
            image.get_width(),
            image.get_height(),
            ColorType::Rgba8,
        )
        .context("Failed to jpeg encode image")?;
    // Set the cursor position to the beginning of the buffer.
    cursor.set_position(0);

    // Read the image from the cursor and convert back to RGBA
    let image = image::io::Reader::new(cursor)
        .with_guessed_format()
        .context("Failed to read image")?
        .decode()?
        .to_rgba8();
    let image = PhotonImage::new(image.to_vec(), image.width(), image.height());
    Ok(image)
}

fn f32ratio_amount(amount: &str) -> Result<(f32, f32)> {
    let (a, b) = amount
        .split_once(':')
        .ok_or_else(|| anyhow!("ratio did not contain two parts"))?;
    let a = a.parse::<f32>()?;
    let b = b.parse::<f32>()?;
    if a.is_nan() || a.is_infinite() {
        Err(anyhow!("a was nan or infinite"))
    } else if b.is_nan() || b.is_infinite() {
        Err(anyhow!("b was nan or infinite"))
    } else {
        Ok((a, b))
    }
}
