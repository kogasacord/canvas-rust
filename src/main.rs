use std::{path::Path, cmp, collections::HashMap, sync::{Arc, Mutex, RwLock}, sync::mpsc};

use awc::Client;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, web::{self, Bytes}, middleware, http::header::AcceptEncoding, Error};
use ril::{Image, Paste, TextAlign, Font, TextLayout, WrapStyle, TextSegment, Rgba, ResizeAlgorithm, OverlayMode, ImageFormat};
use serde::{Serialize, Deserialize};

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok()
        .body(req_body)
}

#[derive(Serialize, Deserialize)]
struct QuoteFnParams {
    author: String,
    text: String,
    png_url: String,
}

#[derive(Serialize, Deserialize)]
struct ChessFnParams {
    fen: String,
}

#[post("/quote_fn")]
async fn quote_fn(data: web::Data<Arc<AppState>>, req_body: web::Json<QuoteFnParams>) -> impl Responder {
    let mut quote = data.quote.lock().unwrap();

    let client = Client::default();
    match client.get(&req_body.png_url).send().await {
        Ok(mut bytes) => {
            match bytes.body().limit(10_000_000).await {
                Ok(bytes) => {
                    if let Err(err) = quote.replace_pfp(bytes) {
                        return HttpResponse::BadRequest().body(err.to_string());
                    }
                },
                Err(err) => {
                    return HttpResponse::BadRequest().body(err.to_string());
                }
            }
        },
        Err(err) => { 
            return HttpResponse::BadRequest().body(err.to_string()) 
        }
    }

    match create_quote(&mut quote, &req_body.text, &req_body.author) {
        Ok(img) => {
            HttpResponse::Ok()
                .body(img)
        },
        Err(err) => {
            HttpResponse::BadRequest()
                .body(err.to_string())
        }
    }
}

#[post("/chess")]
async fn chess_fn(data: web::Data<Arc<AppState>>, req_body: web::Json<ChessFnParams>) -> impl Responder {
    let assets = data.chess_assets.read().unwrap();
    let image_bytes = fen_to_board_img(&req_body.fen, 4, &assets.0, &assets.1);

    HttpResponse::Ok()
        .body(image_bytes)
}


struct AppState {
    quote: Mutex<Quote>,
    chess_assets: RwLock<(HashMap<char, Image<Rgba>>, Image<Rgba>)>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let quote = Mutex::new(Quote::new("./media/images/overlay_gradient.png", "./media/images/profile.png").unwrap());
    let chess_assets = RwLock::new(load_piece_board_images().unwrap());
    let state = web::Data::new(Arc::new(AppState { quote, chess_assets }));
    HttpServer::new(move || {
        App::new()
            .service(quote_fn)
            .service(chess_fn)
            .service(echo)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 4000))?
    .workers(4)
    .run()
    .await?;

    Ok(())
}

/**
 * https://www.desmos.com/calculator/uugepxxtlc
 */
fn font_size_estimate(text: &str) -> f32 {
    let constrained_len = cmp::max(cmp::min(text.len(), 280), 1);
    -(constrained_len as f32 * 0.16) + 70.0
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
    fn replace_pfp(&mut self, bytes: Bytes) -> ril::Result<()> {
        let mut pfp = Image::<Rgba>::from_bytes_inferred(bytes)?;

        let aspect_ratio = pfp.width() as f32 / pfp.height() as f32;
        let new_height = self.image.height();
        let new_width = (new_height as f32 * aspect_ratio).round() as u32;
        pfp.resize(new_width, new_height, ResizeAlgorithm::Lanczos3);

        self.pfp = pfp;
        Ok(())
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

    let font = quote.font.clone();

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
        .with_position(x + 450, max_y)
        .with_segment(
            &TextSegment::<Rgba>::new(&font, format!("- {}", author), Rgba::white())
                .with_size(27.0));

    quote.combine_pfp_and_gradient();

    quote.get_underlying_image().draw(&text_layout);
    quote.get_underlying_image().draw(&author_layout);

    let resized = quote.get_underlying_image()
        .clone()
        .resized(853, 480, ResizeAlgorithm::Hamming);
    resized.encode(ImageFormat::WebP, &mut bytes).unwrap();
    resized.save_inferred("./test-quote.webp").unwrap();

    Ok(bytes)
}
fn load_piece_board_images() -> ril::Result<(HashMap<char, Image<Rgba>>, Image<Rgba>)> {
    let piece_images: HashMap<char, Image<Rgba>> = [
        // Black Pieces
        (
            'p',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_pawn.png"))?,

        ),
        (
            'r',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_rook.png"))?,

        ),
        (
            'n',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_knight.png"))?,

        ),
        (
            'b',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_bishop.png"))?,

        ),
        (
            'q',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_queen.png"))?,
        ),
        (
            'k',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/black_king.png"))?,

        ),
        // White Pieces
        (
            'P',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_pawn.png"))?,
        ),
        (
            'R',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_rook.png"))?,
        ),
        (
            'N',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_knight.png"))?,
        ),
        (
            'B',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_bishop.png"))?,
        ),
        (
            'Q',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_queen.png"))?,
        ),
        (
            'K',
            Image::<Rgba>::open(Path::new("./media/chess_assets/pieces/white_king.png"))?,
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let board_image = Image::<Rgba>::open(Path::new("./media/chess_assets/board/board.png"))?;
    Ok((piece_images, board_image))
}

fn fen_to_board_img(fen: &str, upscale_multiplier: u32, piece_images: &HashMap<char, Image<Rgba>>, board_image: &Image<Rgba>) -> Vec<u8> {
    let mut bytes = Vec::new();

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
            let offsetted_x = (x * square_size + offset + border_size) as u32;
            let offsetted_y = (y * square_size + offset + border_size) as u32;
            
            img.draw(&Paste::new(piece_image)
                       .with_overlay_mode(OverlayMode::Merge)
                       .with_position(offsetted_x, offsetted_y));
        }
        x += 1;
    }


    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let new_height = img.height() * upscale_multiplier;
    let new_width = (new_height as f32 * aspect_ratio).round() as u32;
    img.resize(new_width, new_height, ResizeAlgorithm::Nearest);

    img.save_inferred("./test.webp").unwrap();
    img.encode(ImageFormat::WebP, &mut bytes).unwrap();

    bytes
}
