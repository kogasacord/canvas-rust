
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use ril::{Image, Rgb, ImageFormat, Font, TextLayout, WrapStyle, TextSegment};

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
            HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .service(echo)
            })
            .bind(("127.0.0.1", 8080))?
            .run()
            .await
        },
        Err(err) => {
            println!("{}", err);
            panic!("aaa");
        },
    }
    
}

fn create_image() -> ril::Result<Vec<u8>> {
    let mut bytes = Vec::new();

    let mut image = Image::new(100, 100, Rgb::white());

    let font = Font::open("Arial.ttf", 16.0)?;
    let (x, y) = image.center();
    let layout = TextLayout::new()
        .centered()
        .with_wrap(WrapStyle::Word)
        .with_width(image.width())
        .with_position(x, y)
        .with_segment(&TextSegment::new(&font, "i love kogasa i love youmu i love kogasa", Rgb::black()));
    image.draw(&layout);
    image.flip(); // https://www.youtube.com/watch?v=hYcb854qGx0
    image = image.hue_rotated(10); // https://www.youtube.com/watch?v=hYcb854qGx0

    image.save_inferred("bruhhh.png")?;

    // image.encode(ImageFormat::Png, &mut bytes).unwrap();
    Ok(bytes)
}
