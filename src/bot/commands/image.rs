use clap::Parser;
use eyre::{eyre, Context, ContextCompat, Result};
use image::{DynamicImage, ImageFormat};
use reqwest::{get, Url};
use serenity::client::Context as SContext;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use std::str::FromStr;
use tempfile::tempdir;

#[derive(Debug, Copy, Clone)]
enum Transformation {
    Invert,
    Greyscale,
    Blur(f32),
    Contrast(f32),
}

impl FromStr for Transformation {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "invert" => Ok(Transformation::Invert),
            "greyscale" | "grayscale" => Ok(Transformation::Greyscale),
            s => {
                let (t, amount) = s
                    .split_once('=')
                    .context("Contained incorrect number of parts")?;
                let amount = f32::from_str(amount)?;
                match t {
                    "blur" => Ok(Transformation::Blur(amount)),
                    "contrast" => Ok(Transformation::Contrast(amount)),
                    _ => Err(eyre!("Unknown transformation")),
                }
            }
        }
    }
}

impl Transformation {
    pub(crate) fn apply(self, image: DynamicImage) -> DynamicImage {
        match self {
            Transformation::Invert => {
                let mut image = image;
                image.invert();
                image
            }
            Transformation::Greyscale => image.grayscale(),
            Transformation::Blur(sigma) => image.blur(sigma),
            Transformation::Contrast(c) => image.adjust_contrast(c),
        }
    }
}

#[derive(Debug, Parser)]
struct TransformationOpt {
    #[clap(short)]
    image: Option<String>,
    transformations: Vec<Transformation>,
}

#[group]
#[commands(transform, getpfp, invert, greyscale, blur, contrast)]
pub(crate) struct Image;

#[command]
async fn transform(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    // bit of a hack to get the first argument to be the "program name" so clap doesn't discard it
    let mut to_parse = vec!["transform".to_string()];
    to_parse.extend(args.iter::<String>().collect::<Result<Vec<_>, _>>()?);
    let opt: TransformationOpt = TransformationOpt::try_parse_from(&to_parse)?;

    let mut url = opt.image.unwrap_or_else(|| msg.author.face());
    if url.starts_with('<') && url.ends_with('>') { url = url[1..^1] }
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
            .ok_or_else(|| eyre!("could not find guild"))?;
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
        .send_files(ctx, &files, |m| m.reference_message(msg))
        .await
        .context("Failed to send message")
}
