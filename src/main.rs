use std::time::SystemTime;

use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseWheelDirection, pixels::Color, rect::Rect,
    render::Canvas, video::Window,
};

use freetype as ft;
use texture_atlas::TextureAtlas;

const FONT_PATH: [&str; 2] = [
     "/nix/store/rqs3nbgwi83a1bv3swxy4jjbg5aibfyc-iosevka-15.6.3/share/fonts/truetype/iosevka-regular.ttf",    "/usr/share/fonts/TTF/iosevka-regular.ttc",
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

    let mut text_doc = TextDoc::new(TEXT.to_owned());

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
                Event::MouseWheel {
                    mut y, direction, ..
                } => {
                    if direction == MouseWheelDirection::Flipped {
                        y = -y;
                    }
                    y *= 20;
                    scroll_velocity = y;
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

#[derive(Hash, Eq, PartialEq)]
struct Glyph {
    font_size: usize,
    char: char,
}

mod texture_atlas;

struct TextDoc {
    text: String,
    scroll_offset: i64,
}

impl TextDoc {
    fn new(text: String) -> TextDoc {
        TextDoc {
            text,
            scroll_offset: 0,
        }
    }

    fn scroll(&mut self, amt: i64) {
        self.scroll_offset += amt;
    }

    fn render(&self, texture_atlas: &mut TextureAtlas, canvas: &mut Canvas<Window>) {
        let (width, height) = canvas.output_size().unwrap();
        let width = width as i64;
        let height = height as i64;

        let line_height = texture_atlas.line_height() as i64;
        let (mut x, mut y) = (0, line_height + self.scroll_offset);

        let chars = if self.text[0..3].as_bytes() == [0xef, 0xbb, 0xbf] {
            self.text[3..].chars()
        } else {
            self.text.chars()
        };

        for c in chars {
            let glyph_info = texture_atlas.get(c, 40).unwrap();
            let metrics = glyph_info.metrics;

            if c == '\n' || x + (metrics.horiAdvance / 64) > width as i64 {
                x = 0;
                y += line_height;
            };

            if c == '\n' {
                continue;
            }

            if y - line_height > height {
                break;
            }

            if y >= -line_height {
                let src = Rect::new(
                    glyph_info.x as _,
                    glyph_info.y as _,
                    glyph_info.width as _,
                    glyph_info.height as _,
                );

                let dst = Rect::new(
                    x as i32 + (metrics.horiBearingX / 64) as i32,
                    y as i32 - (metrics.horiBearingY / 64) as i32,
                    glyph_info.width as _,
                    glyph_info.height as _,
                );

                canvas.copy(texture_atlas.texture(), src, dst).unwrap();
            }

            x += metrics.horiAdvance / 64;
        }
    }
}
