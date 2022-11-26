use std::time::SystemTime;

use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseWheelDirection, pixels::Color, rect::Rect,
    render::Canvas, video::Window,
};

use freetype as ft;
use texture_atlas::TextureAtlas;

const FONT_PATH: &str =
    "/nix/store/rqs3nbgwi83a1bv3swxy4jjbg5aibfyc-iosevka-15.6.3/share/fonts/truetype/iosevka-regular.ttf";

const TEXT: &str = include_str!("../small.txt");

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("Text Viewer", 1200, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();

    let lib = ft::Library::init().unwrap();
    let face = lib.new_face(FONT_PATH, 0).unwrap();

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

// How to generate font atlas
// --------------------------
//
//  A Store all the chars in Unicode?
//
//    There are 150k chars in Unicode.
//    At 20px, we get 400px per char approx.
//    150,000 * 400 = 60,000,000
//    Roughly 2000x2000px atlas.
//
//    Won't work well for higher font sizes as we are already at the texture size limit.
//    Could split into multiple textues.
//
//  B For every document, get all the characters in the document
//    and generate the atlas.
//
//    Will have to regenerate the atlas for every text document.
//    Size of the atlas will be smaller.
//
//  C As we process the document, cache the glyph bitmap for every (glyph, font-size)
//
//    Check if (char, font-size) is in cache.
//    If yes, use that
//    else call freetype to draw it and put it in cache
//

// How to scroll
// -------------
//
//  A Generate the whole page at once as a texture
//
//    Could hit the texture size limit very easily.
//
//  B Generate the whole page at once, but split into multiple textures
//
//    High memory usage.
//    Simpler to implement.
//
//  C Generate 3 pages (prev, current, next)
//
//    When scroll down 0.5 page, (prev = current, current = next, next = render())
//    Will cause more redraws than previous, but memory efficient.
//

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

        for c in self.text.chars() {
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
