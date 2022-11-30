use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use crate::texture_atlas::TextureAtlas;

pub struct TextDoc {
    text: String,
    padding: i64,
    scroll_offset: i64,
    max_scroll_offset: i64,
    line_starts: Vec<usize>,
}

impl TextDoc {
    pub fn new(text: String, padding: i64) -> TextDoc {
        TextDoc {
            text,
            padding,
            scroll_offset: 0,
            max_scroll_offset: i64::MAX,
            line_starts: vec![0],
        }
    }

    pub fn scroll(&mut self, amt: i64) {
        if -(self.scroll_offset + amt) >= 0 && -(self.scroll_offset + amt) <= self.max_scroll_offset
        {
            self.scroll_offset += amt;
        }
    }

    pub fn render(&mut self, texture_atlas: &mut TextureAtlas, canvas: &mut Canvas<Window>) {
        let (width, height) = canvas.output_size().unwrap();
        let width = width as i64;
        let height = height as i64;

        let x_start = self.padding;
        let width = width - 2 * self.padding;

        let font_metrics = texture_atlas.font_metrics(40);
        let line_height = font_metrics.line_height;
        let ascender = font_metrics.ascender;
        let descender = font_metrics.descender;

        let (mut x, mut y) = (x_start, ascender + self.scroll_offset + self.padding);

        canvas.set_draw_color(Color::RGB(0x33, 0x33, 0x33));
        canvas
            .draw_line(
                Point::new(x_start as _, y as _),
                Point::new((x_start + width) as _, y as _),
            )
            .unwrap();

        let mut line_no = ((-self.scroll_offset - self.padding - ascender).max(0) as f64
            / line_height as f64)
            .floor() as usize;

        let chars_start_index = *self.line_starts.get(line_no).unwrap_or(&0);

        let chars = if self.text[0..3].as_bytes() == [0xef, 0xbb, 0xbf] {
            self.text[(3 + chars_start_index)..].chars()
        } else {
            self.text[chars_start_index..].chars()
        };

        for (i, c) in chars.enumerate() {
            let glyph_info = texture_atlas.get(c, 40).unwrap();
            let metrics = glyph_info.metrics;

            if c == '\n' || x + (metrics.horiAdvance / 64) > width as i64 {
                x = x_start;
                y += line_height;
                line_no += 1;

                if self.line_starts.len() == line_no {
                    self.line_starts.push(i);
                } else if self.line_starts.len() < line_no {
                    unreachable!();
                }

                canvas.set_draw_color(Color::RGB(0x33, 0x33, 0x33));
                canvas
                    .draw_line(
                        Point::new(x_start as _, y as _),
                        Point::new((x_start + width) as _, y as _),
                    )
                    .unwrap();
            };

            if c == '\n' || c == '\r' {
                continue;
            }

            if y > height + line_height {
                break;
            }

            if y + line_height >= 0 {
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

        if y + descender + self.padding < height {
            self.max_scroll_offset = -self.scroll_offset;
        }
    }
}
