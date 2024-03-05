use std::{path::Path, cmp};

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use ril::{Image, Paste, TextAlign, Font, TextLayout, WrapStyle, TextSegment, Rgba, ResizeAlgorithm, OverlayMode, Rgb};

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
    let mut quote = Quote::new("./media/images/overlay_gradient.png", "./media/images/profile.png").unwrap();
    match create_quote(&mut quote, "hiiii", "alice") {
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

/**
 * https://www.desmos.com/calculator/4t4e16y73m
 */
fn font_size_estimate(text: &str) -> f32 {
    let constrained_len = cmp::max(cmp::min(text.len(), 130), 1);
    -(constrained_len as f32 * 0.245) + 60.0
}

struct Quote {
    image: Image<Rgba>,
    gradient: Image<Rgba>,
    pfp: Image<Rgba>,
    font: Font,
}
impl Quote {
    fn new(gradient_path: &str, pfp_path: &str) -> ril::Result<Quote> {
        let image = Image::new(1280, 720, Rgba::new(255, 255, 255, 0));
        let gradient = Image::<Rgba>::open(Path::new(gradient_path))?;
        // needs to be changed in order to accept pfps from buffers/Vec<u8>s
        let mut pfp = Image::<Rgba>::open(Path::new(pfp_path))?;
        let font = Font::open("./media/fonts/playfair.ttf", 48.0)?;

        let aspect_ratio = pfp.width() as f32 / pfp.height() as f32;
        let new_height = image.height();
        let new_width = (new_height as f32 * aspect_ratio).round() as u32;
        pfp.resize(new_width, new_height, ResizeAlgorithm::Lanczos3);

        Ok(Self { 
            image, 
            gradient, 
            pfp, 
            font 
        })
    }
    fn combine_pfp_and_gradient(&mut self) {
        self.image.draw(&Paste::new(&self.pfp)
                   .with_overlay_mode(OverlayMode::Merge)
                   .with_position(0, 0));
        self.image.draw(&Paste::new(&self.gradient)
                   .with_overlay_mode(OverlayMode::Merge));
    }
    fn get_underlying_image(&mut self) -> &mut Image<Rgba> {
        &mut self.image
    }
}

fn create_quote(quote: &mut Quote, text: &str, author: &str) -> ril::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let estimated_font_size = font_size_estimate(text);

    let font = Font::open("./media/fonts/playfair.ttf", 48.0)?;
    let (x, y) = quote.get_underlying_image().center();
    let text_layout = TextLayout::<Rgba>::new()
        .centered()
        .with_wrap(WrapStyle::Word)
        .with_width(quote.get_underlying_image().width() / 3)
        .with_align(TextAlign::Center)
        .with_position(x + 350, y)
        .with_segment(
            &TextSegment::<Rgba>::new(&font, text, Rgba::white())
                .with_size(estimated_font_size));

    let (_x, _y, _max_x, max_y) = text_layout.bounding_box();
    let author_layout = TextLayout::<Rgba>::new()
        .with_horizontal_anchor(ril::HorizontalAnchor::Left)
        .with_wrap(WrapStyle::Word)
        .with_width(quote.get_underlying_image().width() / 5)
        .with_align(TextAlign::Right)
        .with_position(x + 400, max_y)
        .with_segment(
            &TextSegment::<Rgba>::new(&font, format!("\n - {}", author), Rgba::white())
                .with_size(24.0));

    quote.combine_pfp_and_gradient();

    quote.get_underlying_image().draw(&text_layout);
    quote.get_underlying_image().draw(&author_layout);

    quote.get_underlying_image().save_inferred("test.png")?;
    // image.encode(ImageFormat::Png, &mut bytes).unwrap();
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