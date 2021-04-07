use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use chrono::prelude::*;
use image::{ImageBuffer, Rgb};
use imageproc::drawing::{draw_text_mut, text_size};
use reqwest::Client;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct Quote {
    id: i32,
    quote: String,
    solution: String,
    author: String,
}

const FONT_BYTES: &'static [u8] = include_bytes!("wqy-microhei.ttc");
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 825;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    loop {
        let request_url = format!("https://phodal.github.io/monitor-api/api.json");
        let client = Client::new();
        let response =
            client.get(&request_url)
                .bearer_auth("")
                .send()
                .await?;
        let quote: Quote = response.json().await?;

        let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([255, 255, 255]));
        draw_image(quote, &mut image);
        let _ = image.save(Path::new("monitor.bmp")).unwrap();

        execute_command();
    }
}

#[cfg(target_os = "macos")]
fn execute_command() {
    sleep(Duration::from_secs(10));
}

#[cfg(target_os = "linux")]
fn execute_command() {
    match std::process::Command::new("sudo")
        .arg("epaper")
        .arg("monitor.bmp")
        .status() {
        Ok(_status) => {}
        Err(err) => {
            println!("{:?}", err);
        }
    }

    sleep(Duration::from_secs(60 * 30));
}

fn draw_image(quote: Quote, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let font = read_font();

    let time_size = 40;
    draw_time(image, &font, time_size);

    let text_size = 80;
    draw_sentence(quote.quote.as_str(), text_size, image, &font, time_size);
    draw_sentence(quote.solution.as_str(), text_size, image, &font, time_size + text_size);
}

fn draw_sentence(text: &str, font_size: u32, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, font: &Font, offset: u32) {
    let main_scale = Scale { x: font_size as f32, y: font_size as f32 };
    let (w, h) = text_size(main_scale, &font, text);
    println!("width: {:?}, height: {:?}", w, h);

    let sub_len: usize = (WIDTH / h as u32) as usize;
    print!("sub_len: {:?}", sub_len);
    let subs = text_to_vec(text, sub_len);
    let mut index = 0;

    for sub in subs {
        let y = index * font_size + offset;
        draw_text_mut(image, Rgb([0u8, 0u8, 0u8]), 0, y, main_scale, &font, sub.as_str());
        index = index + 1;
    }
}

fn draw_time(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, font: &Font, font_size: u32)  {
    let small_scale = Scale { x: font_size as f32, y: font_size as f32 };

    let time = time_now();
    draw_text_mut(image, Rgb([0u8, 0u8, 0u8]), 0, 0, small_scale, &font, time.as_str());
}

fn read_font() -> Font<'static> {
    let font = Vec::from(FONT_BYTES);
    let font = Font::try_from_vec(font).unwrap();
    font
}

fn text_to_vec(origin: &str, sub_len: usize) -> Vec<String> {
    let chars: Vec<char> = origin.chars().collect();
    let subs = &chars.chunks(sub_len)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();

    subs.to_vec()
}

fn time_now() -> String {
    let utc: DateTime<Local> = Local::now();
    let delayed_format = utc.format("%Y-%m-%d %H:%M:%S");

    format!("updated time: {}", delayed_format.to_string())
}
