pub mod body;
pub mod game;
pub mod vec2;

use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    ttf,
    video::Window,
};

pub const SZ_W: i32 = 1024;
pub const SZ_H: i32 = 768;

pub const PLAYER_SPEED: f32 = 0.5;
pub const PLAYER_ROT_SPEED: f32 = 0.1;
pub const GRAVITY: f32 = 0.1;

pub const K_FWD: u32 = 1;
pub const K_BACK: u32 = 2;
pub const K_LEFT: u32 = 4;
pub const K_RIGHT: u32 = 8;
pub const K_TURNLEFT: u32 = 16;
pub const K_TURNRIGHT: u32 = 32;

enum TextPosition {
    Center(i32, i32),
    TLCorner(i32, i32),
}

pub(crate) fn display_text(
    text: &str,
    font: &ttf::Font,
    color: Color,
    position: TextPosition,
    canvas: &mut Canvas<Window>,
) -> Rect {
    let texture_creator = canvas.texture_creator();
    let surf = font.render(text).blended(color).unwrap();
    let texture = texture_creator.create_texture_from_surface(surf).unwrap();
    let TextureQuery { width, height, .. } = texture.query();
    let (x, y) = match position {
        TextPosition::Center(x, y) => (x - width as i32 / 2, y + height as i32 / 2),
        TextPosition::TLCorner(x, y) => (x, y),
    };
    let r = Rect::new(x, y, width, height);
    canvas.copy(&texture, None, r).unwrap();
    return r;
}
