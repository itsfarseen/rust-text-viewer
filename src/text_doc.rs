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
        if (self.scroll_offset + amt) >= 0 && (self.scroll_offset + amt) <= self.max_scroll_offset {
            self.scroll_offset += amt;
        }
    }

    pub fn render(&mut self, texture_atlas: &mut TextureAtlas, canvas: &mut Canvas<Window>) {
        let (width, height) = canvas.output_size().unwrap();
        let width = width as i64;
        let height = height as i64;

        let x_start = self.padding;
        let x_end = width - self.padding;

        let font_metrics = texture_atlas.font_metrics(40);
        let line_height = font_metrics.line_height;
        let ascender = font_metrics.ascender;
        let descender = font_metrics.descender;

        let mut line_no = if self.scroll_offset <= self.padding + ascender + line_height {
            0
        } else {
            1 + (self.scroll_offset - (self.padding + ascender + line_height)) / line_height
        };

        let y_start = if self.scroll_offset <= self.padding + ascender + line_height {
            (self.padding + ascender + line_height) - self.scroll_offset
        } else {
            line_height
                - ((self.scroll_offset - (self.padding + ascender + line_height)) % line_height)
        } - line_height;

        let (mut x, mut y) = (x_start, y_start);

        let line_color = Color::RGB(0x33, 0x33, 0x33);
        draw_line(canvas, line_color, x_start, x_end, y);

        let chars_start_index = *self.line_starts.get(line_no as usize).unwrap_or(&0);

        let chars = if self.text[0..3].as_bytes() == [0xef, 0xbb, 0xbf] {
            self.text[(3 + chars_start_index)..].char_indices()
        } else {
            self.text[chars_start_index..].char_indices()
        };


        for (i0, c) in chars {
            let i = i0 + chars_start_index;

            let glyph_info = texture_atlas.get(c, 40).unwrap();
            let metrics = glyph_info.metrics;

            if c == '\n' || x + (metrics.horiAdvance / 64) > x_end {
                x = x_start;
                y += line_height;
                line_no += 1;

                if self.line_starts.len() == line_no as usize {
                    self.line_starts.push(i + 1);
                } else if self.line_starts.len() < line_no as usize {
                    unreachable!();
                }

                draw_line(canvas, line_color, x_start, x_end, y);

                continue;
            };

            if c == '\r' {
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
            self.max_scroll_offset = self.scroll_offset;
        }
    }
}

fn draw_line(canvas: &mut Canvas<Window>, color: Color, x_start: i64, x_end: i64, y: i64) {
    canvas.set_draw_color(color);
    canvas
        .draw_line(
            Point::new(x_start as _, y as _),
            Point::new(x_end as _, y as _),
        )
        .unwrap();
}
