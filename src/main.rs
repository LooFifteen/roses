use std::{
    io::{BufWriter, Cursor},
    sync::Arc,
};

use ab_glyph::FontRef;
use axum::{
    Router, extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get,
};
use image::{DynamicImage, ImageFormat, ImageReader, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use jiff::{Timestamp, ToSpan, ZonedDateTime, civil::Date};
use tokio::sync::RwLock;

const BACKGROUND: &[u8] = include_bytes!("../assets/background.png");
const FONT: &[u8] = include_bytes!("../assets/font.otf");
const ROSES: Date = Date::constant(2027, 4, 30);
const FONT_SCALE: f32 = 56.0;
const Y: i32 = 130;

type StateType = Arc<RwLock<Option<Cache>>>;

#[derive(Debug, Default)]
struct Cache {
    time: Timestamp,
    data: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .route("/", get(index))
        .with_state(StateType::new(RwLock::new(Some(Cache::default()))));
    let listener = tokio::net::TcpListener::bind("[::]:3000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}

// yes i don't care about error handling
async fn index(State(cache): State<StateType>) -> impl IntoResponse {
    // read from cache
    let read_guard = cache.read().await;
    if let Some(cache) = &*read_guard
        && cache.time + 1.hour() > Timestamp::now()
    {
        return (
            [(CONTENT_TYPE, "image/png".to_string())],
            cache.data.clone(),
        );
    }
    drop(read_guard);

    let image = generate_image().unwrap();
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let data = buffer.into_inner().unwrap().into_inner();

    // write to cache
    *cache.write().await = Some(Cache {
        time: Timestamp::now(),
        data: data.clone(),
    });

    ([(CONTENT_TYPE, "image/png".to_string())], data)
}

fn generate_image() -> anyhow::Result<DynamicImage> {
    let mut image = ImageReader::new(Cursor::new(BACKGROUND))
        .with_guessed_format()?
        .decode()?;

    let font = FontRef::try_from_slice(FONT)?;

    let sleeps = ROSES - ZonedDateTime::now().date();
    let text = &sleeps.get_days().to_string();

    let (text_width, _) = text_size(FONT_SCALE, &font, text);
    let x = image.width() / 2 - text_width / 2 + 8; // +8 lol

    draw_text_mut(
        &mut image,
        Rgba([255, 255, 255, 255]),
        x as i32,
        Y,
        FONT_SCALE,
        &font,
        text,
    );

    Ok(image)
}
