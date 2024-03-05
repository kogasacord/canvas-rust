
use std::{collections::HashMap, path::Path};

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use ril::{Font, GradientBlendMode, Image, PalettedRgba, ResizeAlgorithm, Rgb, Rgba, TextAlign, TextLayout, TextSegment, WrapStyle};

#[get("/ping")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(Vec::<u8>::from([0x1f, 0x8b, 0x08, 0x00, 0xa2, 0x30, 0x10, 0x5c, 0x00, 0x03, 0xcb, 0x48, 0xcd, 0xc9, 0xc9]))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok()
        .body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match create_image() {
        Ok(_) => {
            Ok(())
            /*
            HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .service(echo)
            })
            .bind(("127.0.0.1", 8080))?
            .run()
            .await
            */
        },
        Err(err) => {
            panic!("{}", err);
        },
    }
    
}

fn create_image() -> ril::Result<Vec<u8>> {
    let mut bytes = Vec::new();

    let mut image = Image::new(1280, 720, Rgba::new(255, 255, 255, 0));
    let gradient = Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?;
    let mut pfp = Image::<Rgba>::open(Path::new("./media/images/profile.png"))?;
    let aspect_ratio = pfp.width() as f32 / pfp.height() as f32;
    let new_height = image.height();
    let new_width = (new_height as f32 * aspect_ratio).round() as u32;
    pfp.resize(new_width, new_height, ResizeAlgorithm::Nearest);
    
    let font = Font::open("./media/fonts/arial.ttf", 36.0)?;
    let (x, y) = image.center();
    let layout = TextLayout::<Rgba>::new()
        .centered()
        .with_wrap(WrapStyle::Word)
        .with_width(image.width() / 3)
        .with_align(TextAlign::Center)
        .with_position(x + 350, y)
        .with_segment(
            &TextSegment::<Rgba>::new(&font, "i love kogasa i love youmu i love kogasa i love kogasa i love youmu i love kogasa i love kogasa i love youmu i love kogasa ", Rgba::white())
            .with_size(36.0)
        );

    image.paste(0, 0, &pfp);

    for y in 0..gradient.height() {
        for x in 0..gradient.width() {
            let dest_pixel = image.pixel_mut(x, y);
            let src_pixel = gradient.pixel(x, y);
            let alpha = src_pixel.a as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            dest_pixel.r = (dest_pixel.r as f32 * inv_alpha + src_pixel.r as f32 * alpha) as u8;
            dest_pixel.g = (dest_pixel.g as f32 * inv_alpha + src_pixel.g as f32 * alpha) as u8;
            dest_pixel.b = (dest_pixel.b as f32 * inv_alpha + src_pixel.b as f32 * alpha) as u8;
            dest_pixel.a = (dest_pixel.a as f32 * inv_alpha + src_pixel.a as f32 * alpha) as u8;
        }
    }

    image.draw(&layout);

    image.save_inferred("bruhhh.png")?;
    Ok(bytes)
}

fn load_piece_board_images() -> ril::Result<(HashMap<char, PalettedRgba<'static>>, HashMap<char, PalettedRgba<'static>>)> {
    let piece_images: HashMap<char, PalettedRgba> = [
        // Black Pieces
        (
            'p',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,

        ),
        (
            'r',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,

        ),
        (
            'n',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,

        ),
        (
            'b',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,

        ),
        (
            'q',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'k',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,

        ),
        // White Pieces
        (
            'P',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'R',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'N',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'B',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'Q',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
        (
            'K',
            Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?,
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let board_image =             Image::<Rgba>::open(Path::new("./media/images/overlay_gradient.png"))?;
}





fn fen_to_board_img(fen: &str, save_dir: &str, upscale_multiplier: u32, piece_images: &HashMap<char, RgbaImage>, board_image: &RgbaImage) {
    let board = fen.split_whitespace().next().unwrap();
    let mut img = board_image.clone();
    let square_size = (img.width() - 8) / 8; 
    let piece_size = 16;
    let offset = (square_size - piece_size) / 2;
    let border_size = 4;
    let mut x = 0;
    let mut y = 0;
    for char in board.chars() {
        if char == '/' {
            y += 1;
            x = 0;
            continue;
        }
        if let Some(digit) = char.to_digit(10) {
            x += digit;
            continue;
        }
        if let Some(piece_image) = piece_images.get(&char) {
            overlay(
                &mut img,
                piece_image,
                (x * square_size + offset + border_size) as i64,
                (y * square_size + offset + border_size) as i64,
            );
        }
        x += 1;
    }

    let new_width = img.width() * upscale_multiplier;
    let new_height = img.height() * upscale_multiplier;
    let upscale_filter = image::imageops::FilterType::Nearest;

    let upscaled_img = image::imageops::resize(&img, new_width, new_height, upscale_filter);

    upscaled_img.save(save_dir).unwrap();
}