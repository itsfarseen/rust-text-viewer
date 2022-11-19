use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

const FONT_PATH: &str = "/usr/share/fonts/truetype/iosevka/iosevka-regular.ttf";

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("Text Viewer", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let ttf = sdl2::ttf::init().unwrap();
    let font = ttf.load_font(FONT_PATH, 32).unwrap();
    let text = font
        .render("Hello World")
        .blended_wrapped(Color::BLACK, 200)
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        let src = text.rect();
        canvas
            .copy(
                &text.as_texture(&canvas.texture_creator()).unwrap(),
                src,
                Rect::new(50, 50, src.width(), src.height()),
            )
            .unwrap();
        canvas.present();
        let event = event_pump.wait_event_timeout(1_000u32 / 60);
        if let Some(event) = event {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }
}
