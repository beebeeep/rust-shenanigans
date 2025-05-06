use sdl2::{event::Event, keyboard::Keycode, pixels::Color, ttf};
use std::time::{Duration, Instant};

use roids::*;
const FPS: u64 = 60;

fn main() {
    let fps: Duration = Duration::from_millis(1000 / FPS);
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    sdl_context.mouse().show_cursor(false);
    let window = video
        .window("roids", SZ_W as u32, SZ_H as u32)
        .position_centered()
        .input_grabbed()
        .vulkan()
        .build()
        .unwrap_or_else(|_| {
            video
                .window("roids", SZ_W as u32, SZ_H as u32)
                .position_centered()
                .input_grabbed()
                .opengl()
                .build()
                .unwrap()
        });
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = ttf::init().unwrap();
    let mut font = ttf_context.load_font("font.ttf", 12).unwrap();
    font.set_style(ttf::FontStyle::BOLD);

    let mut game = game::Game::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(k), ..
                } => game.key_down(k),
                Event::KeyUp {
                    keycode: Some(k), ..
                } => game.key_up(k),
                Event::MouseButtonDown {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    println!(
                        "btn down {timestamp} {window_id} {which} {mouse_btn:?}  {clicks} {x} {y}"
                    );
                }
                Event::MouseButtonUp {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    println!(
                        "btn up {timestamp} {window_id} {which} {mouse_btn:?}  {clicks} {x} {y}"
                    );
                }
                Event::MouseMotion { x, y, .. } => game.set_cursor(x, y),
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let start = Instant::now();
        game.progress();
        game.render(&mut canvas, &font);
        canvas.present();

        let elapsed = Instant::now() - start;
        ::std::thread::sleep(fps - elapsed);
    }
}
