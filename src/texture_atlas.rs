use std::collections::HashMap;

use freetype::bitmap::PixelMode;
use freetype::face::LoadFlag;
use freetype::ffi::FT_Glyph_Metrics;
use freetype::Bitmap;
use freetype::Face;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureAccess;
use sdl2::render::TextureCreator;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct Glyph {
    pub char: char,
    pub font_size: u32,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct GlyphInfo {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub metrics: FT_Glyph_Metrics,
}
pub struct TextureAtlas<'r> {
    texture: Texture<'r>,
    size: (u32, u32),
    next: (u32, u32),
    max_height: u32,
    glyphs: HashMap<Glyph, GlyphInfo>,
    face: Face,
}

impl<'r> TextureAtlas<'r> {
    pub fn new<T: 'r>(
        texture_creator: &'r TextureCreator<T>,
        size: (u32, u32),
        face: Face,
    ) -> Self {
        let mut texture = texture_creator
            .create_texture(
                PixelFormatEnum::RGBA8888,
                TextureAccess::Static,
                size.0,
                size.1,
            )
            .unwrap();
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        TextureAtlas {
            texture,
            size,
            next: (0, 0),
            max_height: 0,
            glyphs: HashMap::new(),
            face,
        }
    }

    pub fn texture(&self) -> &Texture<'r> {
        &self.texture
    }

    pub fn line_height(&self, font_size: u32) -> i64 {
        self.face
            .set_char_size((font_size * 64) as isize, 0, 50, 0)
            .unwrap();
        self.face.size_metrics().unwrap().height / 64
    }

    pub fn get(&mut self, char: char, font_size: u32) -> Option<GlyphInfo> {
        if let Some(glyph) = self.glyphs.get(&Glyph { char, font_size }) {
            Some(*glyph)
        } else {
            self.face
                .set_char_size((font_size * 64) as isize, 0, 50, 0)
                .unwrap();
            self.face
                .load_char(char as usize, LoadFlag::RENDER)
                .unwrap();
            let glyph = self.face.glyph();
            let metrics = glyph.metrics();
            let bitmap = glyph.bitmap();
            let bitmap_rgba8888 = BitmapRGBA8888::new(&bitmap);
            let (width, height) = (bitmap_rgba8888.width, bitmap_rgba8888.height);
            let (x, y) = self.get_slot(width, height)?;
            let rect = Rect::new(x as _, y as _, width, height);

            self.texture
                .update(
                    rect,
                    &bitmap_rgba8888.pixel_data,
                    bitmap_rgba8888.pitch as _,
                )
                .unwrap();

            let glyph_info = GlyphInfo {
                x,
                y,
                width,
                height,
                metrics,
            };

            self.glyphs.insert(Glyph { char, font_size }, glyph_info);
            Some(glyph_info)
        }
    }

    fn get_slot(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let (mut x, mut y) = self.next;
        if x + width > self.size.0 {
            x = 0;
            y += self.max_height;
            self.max_height = 0;
        }

        if y + height > self.size.1 {
            return None;
        }

        self.next = (x + width, y);
        self.max_height = self.max_height.max(height);
        Some((x, y))
    }
}

struct BitmapRGBA8888 {
    pixel_data: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32,
}

impl BitmapRGBA8888 {
    pub fn new(bitmap: &Bitmap) -> Self {
        let width = (bitmap.width() as u32).max(1);
        let height = (bitmap.rows() as u32).max(1);
        let bytes_per_pixel = 4 as u32;
        let pitch = width * bytes_per_pixel;

        let mut pixel_data = vec![0u8; (width * height * bytes_per_pixel) as usize];

        let mut draw = |x: u32, y: u32, color_rgba: (u8, u8, u8, u8)| {
            pixel_data[(y * width * bytes_per_pixel + x * bytes_per_pixel + 0) as usize] =
                color_rgba.0;
            pixel_data[(y * width * bytes_per_pixel + x * bytes_per_pixel + 1) as usize] =
                color_rgba.1;
            pixel_data[(y * width * bytes_per_pixel + x * bytes_per_pixel + 2) as usize] =
                color_rgba.2;
            pixel_data[(y * width * bytes_per_pixel + x * bytes_per_pixel + 3) as usize] =
                color_rgba.3;
        };

        if bitmap.pitch() < 0 {
            unimplemented!("Negative pitch not impl'd");
        }

        match bitmap.pixel_mode().unwrap() {
            PixelMode::None => unreachable!(),
            PixelMode::Mono => {
                println!("PixelMode::Mono");
                let mut i = 0;
                let mut y = 0;
                let black = (0, 0, 0, 1);
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() / 8 {
                        let b = bitmap.buffer()[(i + x) as usize];
                        let pixels = [
                            b >> 7 == 1,
                            b >> 6 == 1,
                            b >> 5 == 1,
                            b >> 4 == 1,
                            b >> 3 == 1,
                            b >> 2 == 1,
                            b >> 1 == 1,
                            b >> 0 == 1,
                        ];
                        for (j, p) in pixels.iter().enumerate() {
                            if *p {
                                draw(x as u32 + j as u32, y, black);
                            }
                        }
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
            PixelMode::Gray => {
                println!("PixelMode::Gray");
                let mut i = 0;
                let mut y = 0;
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() {
                        let p = bitmap.buffer()[(i + x) as usize];
                        let color_rgba = (p, p, p, p);
                        draw(x as _, y, color_rgba);
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
            PixelMode::Gray2 => {
                println!("PixelMode::Gray2");
                let mut i = 0;
                let mut y = 0;
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() / 4 {
                        let b = bitmap.buffer()[(i + x) as usize];
                        let pixels = [b >> 6 & 0b11, b >> 4 & 0b11, b >> 2 & 0b11, b >> 0 & 0b11];
                        for (j, p) in pixels.iter().enumerate() {
                            let gray = p * 255 / 3;
                            let color_rgba = (gray, gray, gray, gray);
                            draw(x as u32 + j as u32, y, color_rgba)
                        }
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
            PixelMode::Gray4 => {
                println!("PixelMode::Gray4");
                let mut i = 0;
                let mut y = 0;
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() / 2 {
                        let b = bitmap.buffer()[(i + x) as usize];
                        let pixels = [b >> 4 & 0b1111, b >> 0 & 0b1111];
                        for (j, p) in pixels.iter().enumerate() {
                            let gray = p * 255 / 15;
                            let color_rgba = (gray, gray, gray, gray);
                            draw(x as u32 + j as u32, y, color_rgba)
                        }
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
            PixelMode::Lcd => {
                println!("PixelMode::Lcd");
                let mut i = 0;
                let mut y = 0;
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() {
                        let (r, g, b) = (
                            (i + 3 * x) as usize,
                            (i + 3 * x + 1) as usize,
                            (i + 3 * x + 2) as usize,
                        );
                        let color_rgba = (
                            bitmap.buffer()[r],
                            bitmap.buffer()[g],
                            bitmap.buffer()[b],
                            255,
                        );
                        draw(x as _, y, color_rgba);
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
            PixelMode::LcdV => unimplemented!(),
            PixelMode::Bgra => {
                println!("PixelMode::Bgra");
                let mut i = 0;
                let mut y = 0;
                while (i as usize) < bitmap.buffer().len() {
                    for x in 0..bitmap.width() {
                        let (b, g, r, a) = (
                            (i + 4 * x) as usize,
                            (i + 4 * x + 1) as usize,
                            (i + 4 * x + 2) as usize,
                            (i + 4 * x + 3) as usize,
                        );
                        let color_rgba = (
                            bitmap.buffer()[r],
                            bitmap.buffer()[g],
                            bitmap.buffer()[b],
                            bitmap.buffer()[a],
                        );
                        draw(x as _, y, color_rgba);
                    }
                    i += bitmap.pitch();
                    y += 1;
                }
            }
        };
        Self {
            pixel_data,
            width,
            height,
            pitch,
        }
    }
}
