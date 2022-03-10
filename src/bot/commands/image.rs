use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use reqwest::{get, Url};
use serenity::client::Context as SContext;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use std::collections::VecDeque;
use std::str::FromStr;
use tempfile::tempdir;

#[derive(Debug, Copy, Clone)]
enum Transformation {
    Invert,
    Greyscale,
    Fliph,
    Flipv,
    Blur(f32),
    Contrast(f32),
    Huerotate(i32),
    Brighten(i32),
    Resize((f32, f32)),
    Unsharpen((f32, i32)),
}

impl FromStr for Transformation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "invert" => Ok(Transformation::Invert),
            "greyscale" | "grayscale" => Ok(Transformation::Greyscale),
            "fliph" | "flipx" => Ok(Transformation::Fliph),
            "flipv" | "flipy" => Ok(Transformation::Flipv),
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
                        .split_once(':')
                        .ok_or_else(|| anyhow!("ratio did not contain two parts"))?;
                    let a = f32::from_str(a)?;
                    let b = i32::from_str(b)?;
                    if a.is_nan() || a.is_infinite() {
                        Err(anyhow!("a was nan or infinite"))
                    } else {
                        Ok((a, b))
                    }
                }

                match t {
                    "blur" => Ok(Transformation::Blur(f32::from_str(amount)?)),
                    "contrast" => Ok(Transformation::Contrast(f32::from_str(amount)?)),
                    "huerotate" => Ok(Transformation::Huerotate(i32::from_str(amount)?)),
                    "brighten" => Ok(Transformation::Brighten(i32::from_str(amount)?)),
                    "resize" => Ok(Transformation::Resize(f32ratio_amount(amount)?)),
                    "unsharpen" => Ok(Transformation::Unsharpen(f32i32pair_amount(amount)?)),
                    _ => Err(anyhow!("Unknown transformation")),
                }
            }
        }
    }
}

impl Transformation {
    pub(crate) fn apply(self, mut image: DynamicImage) -> DynamicImage {
        match self {
            Transformation::Invert => {
                image.invert();
                image
            }
            Transformation::Fliph => image.fliph(),
            Transformation::Flipv => image.flipv(),
            Transformation::Greyscale => image.grayscale(),
            Transformation::Blur(sigma) => image.blur(sigma),
            Transformation::Contrast(c) => image.adjust_contrast(c),
            Transformation::Huerotate(value) => image.huerotate(value),
            Transformation::Brighten(value) => image.brighten(value),
            Transformation::Resize((a, b)) => {
                let nwidth = (image.width() as f32 * a) as u32;
                let nheight = (image.height() as f32 * b) as u32;
                image.resize_exact(nwidth, nheight, FilterType::CatmullRom)
            },
            Transformation::Unsharpen((s, t)) => image.unsharpen(s, t),
        }
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
#[commands(transform, getpfp, invert, greyscale, blur, contrast)]
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
    let (format, mut image) = download_image(url).await?;

    image = opt
        .transformations
        .into_iter()
        .fold(image, |i, t| t.apply(i));

    respond_with_image(ctx, msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn getpfp(ctx: &SContext, msg: &Message) -> CommandResult {
    let u = msg.author.face();
    msg.reply(ctx, u).await?;
    Ok(())
}

#[command]
async fn invert(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    image.invert();

    respond_with_image(ctx, msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn greyscale(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    image = image.grayscale();

    respond_with_image(ctx, msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn blur(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    let reaction = msg.react(ctx, '🕐').await?;
    let w = image.width() as f32;
    image = image.blur(w / 40.0);

    reaction.delete(ctx).await?;
    respond_with_image(ctx, msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn contrast(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    image = image.adjust_contrast(100.0);

    respond_with_image(ctx, msg, format, &mut image).await?;
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

async fn download_image(url: String) -> Result<(ImageFormat, DynamicImage)> {
    let format = get_format(&url)?;
    let response = get(url).await?;
    let bytes = response.bytes().await?;
    let image = if format == ImageFormat::WebP {
        webp::Decoder::new(&bytes)
            .decode()
            .context("Failed to load WebP image")?
            .to_image()
    } else {
        image::load_from_memory_with_format(&bytes, format).context("")?
    };
    Ok((format, image))
}

async fn parse_user_avatar(
    ctx: &SContext,
    msg: &Message,
    args: &mut Args,
) -> Result<(ImageFormat, DynamicImage)> {
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
    format: ImageFormat,
    image: &mut DynamicImage,
) -> Result<Message> {
    let extension = get_extension(format);
    let name = format!("{}.{}", &msg.author.name, extension);
    let dir = tempdir()?;
    let file = dir.path().join(&name);
    image.save(&file)?;

    let files = vec![file];
    msg.channel_id
        .send_files(ctx, &files, |m| {
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_users())
        })
        .await
        .context("Failed to send message")
}
