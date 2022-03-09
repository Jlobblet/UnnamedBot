use eyre::{eyre, Context, ContextCompat, Result};
use image::{DynamicImage, ImageFormat};
use reqwest::{get, Url};
use serenity::client::Context as SContext;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use tempfile::tempdir;

#[group]
#[commands(getpfp, invert, greyscale, blur, contrast)]
pub(crate) struct Image;

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

    respond_with_image(ctx, &msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn greyscale(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    image = image.grayscale();

    respond_with_image(ctx, &msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn blur(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    let reaction = msg.react(ctx, '🕐').await?;
    let w = image.width() as f32;
    image = image.blur(w / 40.0);

    reaction.delete(ctx).await?;
    respond_with_image(ctx, &msg, format, &mut image).await?;
    Ok(())
}

#[command]
async fn contrast(ctx: &SContext, msg: &Message, mut args: Args) -> CommandResult {
    let (format, mut image) = parse_user_avatar(ctx, msg, &mut args).await?;
    image = image.adjust_contrast(1.5);

    respond_with_image(ctx, &msg, format, &mut image).await?;
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

async fn download_avatar(user: &User) -> Result<(ImageFormat, DynamicImage)> {
    let url = user.face();
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
    download_avatar(&user).await
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
