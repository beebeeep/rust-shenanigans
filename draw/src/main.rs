use std::{thread, time::Duration};

use hsl::HSL;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut events = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("draw", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.present();

    let mut tick = 0i64;
    'run: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'run,
                _ => {}
            }
        }

        let mut a = 1f32;
        for d in 0..100 {
            let t = (tick as f32 - (d as f32 * 0.2)).to_radians();
            a *= 0.95;
            let r = 200.0;
            let (x, y) = (r * f32::cos(t), r * f32::sin(t));
            canvas.set_draw_color(Color::RGBA(0x10, 0xa0, 0x10, (255.0 * a) as u8));
            canvas
                .draw_line(
                    (WIDTH / 2, HEIGHT / 2),
                    (WIDTH / 2 + x as i32, HEIGHT / 2 + y as i32),
                )
                .unwrap();
        }
        canvas.present();
        tick += 1;
        thread::sleep(Duration::from_millis(8));
    }
}
