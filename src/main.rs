
use std::path::Path;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use ril::{Image, Rgb, TextAlign, Font, TextLayout, WrapStyle, TextSegment, Rgba};

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
    let pfp = Image::<Rgba>::open(Path::new("./media/images/profile.png"))?;

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
    image.paste(0, 0, &gradient);
    image.draw(&layout);

    image.save_inferred("bruhhh.png")?;
    // image.encode(ImageFormat::Png, &mut bytes).unwrap();
    Ok(bytes)
}
