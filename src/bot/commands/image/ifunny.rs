use photon_rs::multiple::watermark;
use photon_rs::native::open_image;
use photon_rs::transform::padding_bottom;
use photon_rs::{PhotonImage, Rgba};

fn add_watermark(mut image: PhotonImage, path: &str) -> PhotonImage {
    let wmark = open_image(path).expect("Failed to open watermark");

    let black = Rgba::new(0, 0, 0, 255);
    let y = image.get_height();
    let x = image.get_width() - wmark.get_width();
    image = padding_bottom(&image, wmark.get_height(), black);
    watermark(&mut image, &wmark, x, y);
    image
}

pub(super) fn add_ifunny_watermark(image: PhotonImage) -> PhotonImage {
    add_watermark(image, "resources/ifunny.png")
}
