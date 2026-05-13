//! Bitmap text using font8x8 (no SDL2_ttf).

use font8x8::UnicodeFonts;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Draw text with 8x8 BASIC font glyphs. Scale multiplies pixel size.
pub fn draw_text_utf8(
    canvas: &mut Canvas<Window>,
    fonts: &impl UnicodeFonts,
    text: &str,
    mut x: i32,
    y: i32,
    scale: u32,
    color: Color,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    let s = scale.max(1);
    for ch in text.chars() {
        if let Some(glyph) = fonts.get(ch) {
            // font8x8: bit 0 = left column (see dhepper/font8x8 convention).
            for (row, &byte) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if byte & (1 << col) != 0 {
                        let px = x + col as i32 * s as i32;
                        let py = y + row as i32 * s as i32;
                        canvas.fill_rect(Rect::new(px, py, s, s))?;
                    }
                }
            }
        }
        x += (8 * s) as i32;
    }
    Ok(())
}
