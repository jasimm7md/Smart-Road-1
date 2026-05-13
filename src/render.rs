//! Road surface + inbound labels/arrows per objectives mapping (North/South/West/East row/column rules).

use crate::intersection::IntersectionConfig;
use crate::simple_text;
use font8x8::BASIC_FONTS;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::Window;
use std::path::Path;

pub struct Assets<'a> {
    pub car: Texture<'a>,
    pub road: Texture<'a>,
}

pub fn load_texture<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    path: impl AsRef<Path>,
) -> Result<Texture<'a>, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = img.dimensions();
    let mut texture = texture_creator
        .create_texture_static(PixelFormatEnum::ABGR8888, w, h)
        .map_err(|e| e.to_string())?;

    texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    texture.update(None, img.as_raw(), (w * 4) as usize).map_err(|e| e.to_string())?;
    Ok(texture)
}

const VOID_BG: Color = Color::RGB(40, 44, 52);
const ROAD_ASPHALT: Color = Color::RGB(72, 74, 78);
const ROAD_EDGE: Color = Color::RGB(200, 200, 205);
const LANE_LINE: Color = Color::RGB(235, 235, 240);
const CENTER_DASH: Color = Color::RGB(248, 220, 90);
const LABEL: Color = Color::RGB(160, 170, 185);
const ARROW: Color = Color::RGB(190, 200, 215);

fn fill_rect(canvas: &mut Canvas<Window>, x: i32, y: i32, w: u32, h: u32, c: Color) -> Result<(), String> {
    canvas.set_draw_color(c);
    canvas.fill_rect(Rect::new(x, y, w, h))?;
    Ok(())
}

fn dashed_vline(
    canvas: &mut Canvas<Window>,
    x: i32,
    y0: i32,
    y1: i32,
    dash: i32,
    gap: i32,
    c: Color,
) -> Result<(), String> {
    let mut y = y0.min(y1);
    let end = y0.max(y1);
    canvas.set_draw_color(c);
    while y < end {
        let yd = (y + dash).min(end);
        canvas.fill_rect(Rect::new(x - 1, y, 3, (yd - y) as u32))?;
        y = yd + gap;
    }
    Ok(())
}

fn dashed_hline(
    canvas: &mut Canvas<Window>,
    y: i32,
    x0: i32,
    x1: i32,
    dash: i32,
    gap: i32,
    c: Color,
) -> Result<(), String> {
    let mut x = x0.min(x1);
    let end = x0.max(x1);
    canvas.set_draw_color(c);
    while x < end {
        let xd = (x + dash).min(end);
        canvas.fill_rect(Rect::new(x, y - 1, (xd - x) as u32, 3))?;
        x = xd + gap;
    }
    Ok(())
}

fn solid_vline(canvas: &mut Canvas<Window>, x: i32, y0: i32, y1: i32, c: Color) -> Result<(), String> {
    canvas.set_draw_color(c);
    let y = y0.min(y1);
    let h = (y0.max(y1) - y) as u32;
    canvas.fill_rect(Rect::new(x - 1, y, 3, h.max(1)))?;
    Ok(())
}

fn solid_hline(canvas: &mut Canvas<Window>, y: i32, x0: i32, x1: i32, c: Color) -> Result<(), String> {
    canvas.set_draw_color(c);
    let x = x0.min(x1);
    let w = (x0.max(x1) - x) as u32;
    canvas.fill_rect(Rect::new(x, y - 1, w.max(1), 3))?;
    Ok(())
}

/// Draw roads, lane dividers, median dashed lines (between inbound / outbound), inbound r/s/l labels.
pub fn draw_road_and_markings(
    canvas: &mut Canvas<Window>,
    inter: &IntersectionConfig,
    assets: &Assets,
    window_w: u32,
    window_h: u32,
) -> Result<(), String> {
    let cx = inter.center_x as i32;
    let cy = inter.center_y as i32;
    let h = inter.half_size as i32;
    let lw = inter.lane_width as i32;
    let road_w = (inter.lane_width * 6.0) as i32;
    let half_road = road_w / 2;

    // Inbound lane centers (3 per approach half), matching intersection geometry.
    let in_x = |i: i32| cx + ((-5 + i * 2) * lw) / 2;
    let in_y = |i: i32| cy + ((-5 + i * 2) * lw) / 2;
    let in_x_s = |i: i32| cx + ((1 + i * 2) * lw) / 2;
    let in_y_e = |i: i32| cy + ((1 + i * 2) * lw) / 2;

    fill_rect(canvas, 0, 0, window_w, window_h, VOID_BG)?;

    // Draw road texture tiled instead of solid asphalt
    canvas.set_draw_color(ROAD_ASPHALT); // fallback
    for x in (cx - half_road..cx + half_road).step_by(64) {
        for y in (0..window_h as i32).step_by(64) {
            let _ = canvas.copy(&assets.road, None, Rect::new(x, y, 64, 64));
        }
    }
    for y in (cy - half_road..cy + half_road).step_by(64) {
        for x in (0..window_w as i32).step_by(64) {
            let _ = canvas.copy(&assets.road, None, Rect::new(x, y, 64, 64));
        }
    }

    canvas.set_draw_color(Color::RGB(58, 60, 64));
    canvas.fill_rect(Rect::new(
        cx - h,
        cy - h,
        (h * 2) as u32,
        (h * 2) as u32,
    ))?;

    let edge = 2u32;
    canvas.set_draw_color(ROAD_EDGE);
    canvas.fill_rect(Rect::new(cx - half_road - 2, 0, edge, window_h))?;
    canvas.fill_rect(Rect::new(cx + half_road, 0, edge, window_h))?;
    canvas.fill_rect(Rect::new(0, cy - half_road - 2, window_w, edge))?;
    canvas.fill_rect(Rect::new(0, cy + half_road, window_w, edge))?;

    // Solid lane lines only (median at cx/cy is dashed yellow, not white).
    for k in [-2, -1, 1, 2] {
        let x = cx + k * lw;
        solid_vline(canvas, x, 0, window_h as i32, LANE_LINE)?;
    }

    let dash = 10;
    let gap = 8;
    dashed_vline(canvas, cx, 0, window_h as i32, dash, gap, CENTER_DASH)?;
    dashed_hline(canvas, cy, 0, window_w as i32, dash, gap, CENTER_DASH)?;

    // Horizontal lane dividers (E–W road): draw after median so they are not covered, and
    // split into left arm | intersection | right arm so left/right wings always show 3+3 lanes.
    let w = window_w as i32;
    let xl = cx - half_road;
    let xr = cx + half_road;
    for k in [-2, -1, 1, 2] {
        let y = cy + k * lw;
        solid_hline(canvas, y, 0, xl, LANE_LINE)?;
        solid_hline(canvas, y, xl, xr, LANE_LINE)?;
        solid_hline(canvas, y, xr, w, LANE_LINE)?;
    }

    let scale = 1u32;
    let line = 10i32;
    // Layout matches objectives/README mapping (letters vs arrows per side).
    let gap = 14i32;
    let north_letter_y = cy - h - gap - 8;
    let north_arrow_y = north_letter_y + line;
    // South: 1st row arrows, 2nd row letters
    let south_arrow_y = cy + h + 6;
    let south_letter_y = south_arrow_y + line;
    let west_x = cx - h - 22;
    let east_x = cx + h + 8;
    // West: col1 letter, col2 space, col3 arrow — East: col1 arrow, col2 space, col3 letter (8px glyph + 8px gap)
    let col3 = 16i32;

    // North: line 1 letters, line 2 arrows — r ←, s ↓, l →
    for (i, ch, ar) in [(0, "r", "<"), (1, "s", "v"), (2, "l", ">")] {
        let x = in_x(i) - 4;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ch, x, north_letter_y, scale, LABEL)?;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ar, x, north_arrow_y, scale, ARROW)?;
    }

    // South: line 1 arrows, line 2 letters — l ←, s ↑, r →
    for (i, ar, ch) in [(0, "<", "l"), (1, "^", "s"), (2, ">", "r")] {
        let x = in_x_s(i) - 4;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ar, x, south_arrow_y, scale, ARROW)?;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ch, x, south_letter_y, scale, LABEL)?;
    }

    // West: col1 letters, col2 space, col3 arrows — l ↑, s →, r ↓
    for (i, ch, ar) in [(0, "l", "^"), (1, "s", ">"), (2, "r", "v")] {
        let y = in_y_e(i) - 4;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ch, west_x, y, scale, LABEL)?;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ar, west_x + col3, y, scale, ARROW)?;
    }

    // East: col1 arrows, col2 space, col3 letters — r ↑, s ←, l ↓
    for (i, ar, ch) in [(0, "^", "r"), (1, "<", "s"), (2, "v", "l")] {
        let y = in_y(i) - 4;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ar, east_x, y, scale, ARROW)?;
        simple_text::draw_text_utf8(canvas, &BASIC_FONTS, ch, east_x + col3, y, scale, LABEL)?;
    }

    Ok(())
}
