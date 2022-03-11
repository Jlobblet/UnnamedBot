use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::{ImageBuffer, ImageFormat, RgbaImage, RgbImage};
use reqwest::{get, Url};
use serenity::client::Context as SContext;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use std::collections::VecDeque;
use std::str::FromStr;
use image::DynamicImage::ImageRgb8;
use log::warn;
use photon_rs::channels::invert;
use photon_rs::colour_spaces::hue_rotate_hsv;
use photon_rs::conv::{gaussian_blur, sharpen};
use photon_rs::effects::{adjust_contrast, colorize, frosted_glass, inc_brightness, solarize};
use photon_rs::monochrome::{grayscale, grayscale_human_corrected, monochrome};
use photon_rs::native::save_image;
use photon_rs::{monochrome, PhotonImage};
use photon_rs::noise::add_noise_rand;
use photon_rs::transform::{fliph, flipv, resize, SamplingFilter};
use tempfile::tempdir;
use webp::PixelLayout::Rgb;

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
    Blur(i32),
    Contrast(f32),
    Huerotate(f32),
    Brighten(u8),
    Resize((f32, f32)),
    Sharpen(u8),
}

impl FromStr for Transformation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "invert" => Ok(Transformation::Invert),
            "greyscale" | "grayscale" => Ok(Transformation::Greyscale),
            "fliph" | "flipx" => Ok(Transformation::Fliph),
            "flipv" | "flipy" => Ok(Transformation::Flipv),
            "noise" => Ok(Transformation::Noise),
            "frost" => Ok(Transformation::Frost),
            "solarise" | "solarize" => Ok(Transformation::Solarise),
            "colourise" | "colorize "=> Ok(Transformation::Colourise),
            s => {
                let (t, amount) = s
                    .split_once('=')
                    .with_context(|| anyhow!("{} did not match any of the parameterless verbs and did not contain an `=`", s))?;

                fn f32ratio_amount(amount: &str) -> Result<(f32, f32)> {
                    let (a, b) = amount
                        .split_once(':')
                        .ok_or_else(|| anyhow!("ratio did not contain two parts"))?;
                    let a = f32::from_str(a)?;
                    let b = f32::from_str(b)?;
                    if a.is_nan() || a.is_infinite() {
                        Err(anyhow!("a was nan or infinite"))
                    } else if b.is_nan() || b.is_infinite() {
                        Err(anyhow!("b was nan or infinite"))
                    } else {
                        Ok((a, b))
                    }
                }

                fn f32i32pair_amount(amount: &str) -> Result<(f32, i32)> {
                    let (a, b) = amount
                        .split_once(',')
                        .ok_or_else(|| anyhow!("pair did not contain two parts"))?;
                    let a = f32::from_str(a)?;
                    let b = i32::from_str(b)?;
                    if a.is_nan() || a.is_infinite() {
                        Err(anyhow!("a was nan or infinite"))
                    } else {
                        Ok((a, b))
                    }
                }

                match t {
                    "blur" => Ok(Transformation::Blur(i32::from_str(amount)?)),
                    "contrast" => Ok(Transformation::Contrast(f32::from_str(amount)?)),
                    "huerotate" => Ok(Transformation::Huerotate(f32::from_str(amount)?)),
                    "brighten" => Ok(Transformation::Brighten(u8::from_str(amount)?)),
                    "resize" => Ok(Transformation::Resize(f32ratio_amount(amount)?)),
                    "sharpen" => Ok(Transformation::Sharpen(u8::from_str(amount)?)),
                    _ => Err(anyhow!("Unknown transformation")),
                }
            }
        }
    }
}

impl Transformation {
    pub(crate) fn apply(self, mut image: PhotonImage) -> PhotonImage {
        match self {
            Transformation::Invert => invert(&mut image),
            Transformation::Fliph => fliph(&mut image),
            Transformation::Flipv => flipv(&mut image),
            Transformation::Noise => image = add_noise_rand(image),
            Transformation::Greyscale => grayscale_human_corrected(&mut image),
            Transformation::Frost => frosted_glass(&mut image),
            Transformation::Solarise => solarize(&mut image),
            Transformation::Colourise => colorize(&mut image),
            Transformation::Blur(radius) => gaussian_blur(&mut image, radius),
            Transformation::Contrast(c) => adjust_contrast(&mut image, c),
            Transformation::Huerotate(d) => hue_rotate_hsv(&mut image, d),
            Transformation::Brighten(value) => inc_brightness(&mut image, value),
            Transformation::Resize((a, b)) => {
                let width = (image.get_width() as f32 * a) as u32;
                let height = (image.get_height() as f32 * b) as u32;
                resize(&mut image, width, height, SamplingFilter::CatmullRom);
            },
            Transformation::Sharpen(n) => (0..n).for_each(|_| sharpen(&mut image)),
        }
        image
    }
}

#[derive(Debug, Parser)]
struct TransformationOpt {
    #[clap(short)]
    user: Option<u64>,
    #[clap(short)]
    image: Option<String>,
    transformations: Vec<Transformation>,
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
        opt.image.unwrap_or_else(|| msg.author.face())
    };
    if url.starts_with('<') && url.ends_with('>') {
        let mut chars = url.chars();
        chars.next();
        chars.next_back();
        url = chars.collect();
    }
    let mut image = download_image(url).await?;

    image = opt
        .transformations
        .into_iter()
        .fold(image, |i, t| t.apply(i));

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

fn get_extension(format: ImageFormat) -> &'static str {
    if format.can_write() {
        format.extensions_str()[0]
    } else {
        ImageFormat::Png.extensions_str()[0]
    }
}

async fn download_image(url: String) -> Result<PhotonImage> {
    let format = get_format(&url)
        .context("Could not determine format when attempting to download image")?;
    let response = get(&url).await
        .with_context(|| anyhow!("Failed to get response from {}", &url))?;
    let bytes = response.bytes().await
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

async fn parse_user_avatar(
    ctx: &SContext,
    msg: &Message,
    args: &mut Args,
) -> Result<PhotonImage> {
    let user = if args.is_empty() {
        msg.author.clone()
    } else {
        let id = args.single::<u64>()?;
        let guild = msg
            .guild(ctx)
            .await
            .ok_or_else(|| anyhow!("could not find guild"))?;
        guild.member(ctx, id).await?.user
    };
    let url = user.face();
    download_image(url).await
}

async fn respond_with_image(
    ctx: &SContext,
    msg: &Message,
    filename: &str,
    image: PhotonImage,
) -> Result<Message> {
    let name = format!("{}.png", filename);
    let dir = tempdir()
        .context("Could not create temporary directory")?;
    let file = dir.path().join(&name);

    let image = RgbaImage::from_raw(image.get_width(), image.get_height(), image.get_raw_pixels())
        .context("Failed to load image for saving")?;

    image.save(&file).context("Failed to save image")?;
    // save_image(image, file.to_str().context("Failed to create filepath to save")?);

    let files = vec![file];
    msg.channel_id
        .send_files(ctx, &files, |m| {
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_users())
        })
        .await
        .context("Failed to send message")
}
