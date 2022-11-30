use std::time::SystemTime;

use freetype as ft;
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseWheelDirection, pixels::Color};

mod text_doc;
mod texture_atlas;

use text_doc::TextDoc;
use texture_atlas::TextureAtlas;

const FONT_PATH: [&str; 3] = [
    "/usr/share/fonts/TTF/OpenSans-Light.ttf",
    "/nix/store/rqs3nbgwi83a1bv3swxy4jjbg5aibfyc-iosevka-15.6.3/share/fonts/truetype/iosevka-regular.ttf",
    "/usr/share/fonts/TTF/iosevka-regular.ttc",
];

const TEXT: &str = include_str!("../small.txt");

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("Text Viewer", 1200, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();

    let lib = ft::Library::init().unwrap();

    let mut face = None;
    for f in FONT_PATH {
        if let Ok(face_) = lib.new_face(f, 0) {
            face = Some(face_);
            break;
        }
    }
    if face.is_none() {
        println!("E: Can't load fonts");
        return;
    }
    let face = face.unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture_atlas = TextureAtlas::new(&texture_creator, (1024, 1024), face);

    let mut text_doc = TextDoc::new(TEXT.to_owned(), 40);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_time = SystemTime::now();
    let mut frame = 0.0;
    let mut scroll_velocity = 0;
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        text_doc.render(&mut texture_atlas, &mut canvas);
        canvas.present();
        let event = event_pump.wait_event_timeout(1_000u32 / 120);
        if let Some(event) = event {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    scroll_velocity = 1;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    scroll_velocity = -1;
                }
                Event::MouseWheel {
                    mut y, direction, ..
                } => {
                    if direction == MouseWheelDirection::Flipped {
                        y = -y;
                    }
                    y *= 20;
                    scroll_velocity = y;
                }
                Event::FingerMotion { mut dy, .. } => {
                    dy *= 500.0;
                    scroll_velocity = dy as _;
                }
                _ => {}
            }
        }

        text_doc.scroll(scroll_velocity as _);
        scroll_velocity = scroll_velocity * 999 / 1000;

        frame += 1.0;
        let elapsed = last_time.elapsed().unwrap();
        if elapsed.as_secs() > 1 {
            let fps = frame / elapsed.as_secs_f32();
            println!("FPS: {fps}");
            last_time = SystemTime::now();
            frame = 0.0;
        }
    }
}
