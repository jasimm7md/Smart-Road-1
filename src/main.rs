//! Smart Road: intersection simulation for autonomous vehicles (no traffic lights).
//! Arrow keys spawn vehicles, R = continuous random spawn, Esc = exit and show statistics.

mod controller;
mod intersection;
mod render;
mod simple_text;
mod stats;
mod vehicle;

use font8x8::BASIC_FONTS;
use intersection::{Direction, IntersectionConfig, PathId, Route, path_length};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;
use vehicle::{Vehicle, VehiclePhysicsConfig};

const WINDOW_W: u32 = 800;
const WINDOW_H: u32 = 600;
const TARGET_FPS: u32 = 60;
const DT: f64 = 1.0 / TARGET_FPS as f64;

/// Spawn cooldown per direction (seconds) to prevent vehicles on top of each other.
const SPAWN_COOLDOWN: f64 = 0.8;
const R_RANDOM_INTERVAL: f64 = 0.5;
/// Cap simultaneous vehicles (holding R otherwise spawns hundreds; they stack and slow the sim).
const MAX_VEHICLES: usize = 80;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;
    let window = video
        .window("Smart Road - Intersection", WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.clear();
    canvas.present();

    let inter = IntersectionConfig::default();
    let physics = VehiclePhysicsConfig::default();
    let texture_creator = canvas.texture_creator();
    let car_tex = render::load_texture(&texture_creator, "assets/car.png")?;
    let road_tex = render::load_texture(&texture_creator, "assets/road.png")?;
    let assets = render::Assets {
        car: car_tex,
        road: road_tex,
    };
    let mut stats = stats::StatsCollector::new();
    let mut vehicles: Vec<Vehicle> = Vec::new();
    let mut next_id: u32 = 0;
    // Use negative sentinel so first spawn per direction is allowed immediately (0 - (-0.8) >= 0.8).
    let mut last_spawn: [f64; 4] = [-SPAWN_COOLDOWN; 4]; // N, S, E, W
    let mut r_held = false;
    let mut r_accum: f64 = 0.0;
    let mut event_pump = sdl_context.event_pump()?;
    let mut sim_time: f64 = 0.0;

    'running: loop {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if k == Keycode::Escape {
                        break 'running;
                    }
                    if k == Keycode::R {
                        r_held = true;
                        // First random spawn on key-down without waiting a full interval.
                        r_accum = R_RANDOM_INTERVAL;
                    }
                    // Subject: Up=from south, Down=from north, Right=from west, Left=from east.
                    let dir = match k {
                        Keycode::Up => Some(Direction::South),
                        Keycode::Down => Some(Direction::North),
                        Keycode::Right => Some(Direction::West),
                        Keycode::Left => Some(Direction::East),
                        _ => None,
                    };
                    if let Some(d) = dir {
                        let idx = dir_index(d);
                        if sim_time - last_spawn[idx] >= SPAWN_COOLDOWN
                            && try_spawn_vehicle(d, &mut vehicles, &mut next_id, &inter)
                        {
                            last_spawn[idx] = sim_time;
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => r_held = false,
                _ => {}
            }
        }

        if r_held {
            r_accum += DT;
            if r_accum >= R_RANDOM_INTERVAL {
                r_accum = 0.0;
                let d = random_direction();
                let _ = try_spawn_vehicle(d, &mut vehicles, &mut next_id, &inter);
            }
        }

        sim_time += DT;

        controller::update_velocities(&mut vehicles, &physics, &inter, &mut stats);

        let mut i = 0;
        while i < vehicles.len() {
            let exited = vehicles[i].update(DT, &physics, &inter, &mut stats);
            if exited {
                vehicles.remove(i);
            } else {
                i += 1;
            }
        }

        draw(&mut canvas, &inter, &assets, &vehicles, vehicles.len())?;

        let elapsed = frame_start.elapsed();
        let frame_duration = 1.0 / TARGET_FPS as f64;
        if elapsed.as_secs_f64() < frame_duration {
            std::thread::sleep(std::time::Duration::from_secs_f64(
                frame_duration - elapsed.as_secs_f64(),
            ));
        }
    }

    print_stats_to_console(&stats);
    let _ = show_stats_window(&sdl_context, &stats);
    Ok(())
}

fn print_stats_to_console(stats: &stats::StatsCollector) {
    println!("--- Simulation Statistics ---");
    println!("Max vehicles passed: {}", stats.vehicles_passed);
    println!("Max velocity: {:.1}", stats.max_velocity);
    println!("Min velocity: {:.1}", stats.min_velocity_display());
    println!(
        "Max time through intersection: {:.2} s",
        stats.max_time_through_intersection
    );
    println!(
        "Min time through intersection: {:.2} s",
        stats.min_time_display()
    );
    println!("Close calls: {}", stats.close_calls);
}

fn dir_index(d: Direction) -> usize {
    match d {
        Direction::North => 0,
        Direction::South => 1,
        Direction::East => 2,
        Direction::West => 3,
    }
}

fn random_direction() -> Direction {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => Direction::North,
        1 => Direction::South,
        2 => Direction::East,
        _ => Direction::West,
    }
}

fn random_route() -> Route {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..3) {
        0 => Route::Right,
        1 => Route::Straight,
        _ => Route::Left,
    }
}

/// Returns `true` if a vehicle was added.
fn try_spawn_vehicle(
    direction: Direction,
    vehicles: &mut Vec<Vehicle>,
    next_id: &mut u32,
    inter: &IntersectionConfig,
) -> bool {
    if vehicles.len() >= MAX_VEHICLES {
        return false;
    }
    let path = PathId::from_direction_route(direction, random_route());
    let len = path_length(inter, path);
    let v = Vehicle::new(*next_id, path, len);
    *next_id += 1;
    vehicles.push(v);
    true
}

fn draw(
    canvas: &mut Canvas<Window>,
    inter: &IntersectionConfig,
    assets: &render::Assets,
    vehicles: &[Vehicle],
    vehicle_count: usize,
) -> Result<(), String> {
    render::draw_road_and_markings(canvas, inter, assets, WINDOW_W, WINDOW_H)?;

    for v in vehicles {
        let pos = v.position(inter);
        let x = pos.x as i32;
        let y = pos.y as i32;
        
        let car_tex = &assets.car;
        // Since we can't easily mut assets, we'll just set color mod on canvas if we could, 
        // but Texture needs to be mutable to set color mod. It's fine to just draw it as is,
        // or we can just draw the texture and maybe draw a small indicator if we want. 
        // For now, let's just draw the car texture rotating.
        let angle_deg = v.angle_rad(inter) * 180.0 / std::f64::consts::PI + 90.0;
        let rect = Rect::new(x - 16, y - 32, 32, 64);
        
        canvas.copy_ex(
            car_tex,
            None,
            rect,
            angle_deg,
            None,
            false,
            false,
        )?;
    }

    // On-screen help: SDL only sends keys to the focused window.
    let hint = Color::RGB(180, 190, 200);
    let scale = 1u32;
    let line = 11i32;
    let mut y = WINDOW_H as i32 - 72;
    simple_text::draw_text_utf8(
        canvas,
        &BASIC_FONTS,
        "Click this window first, then:",
        8,
        y,
        scale,
        hint,
    )?;
    y += line;
    simple_text::draw_text_utf8(
        canvas,
        &BASIC_FONTS,
        "Arrows = spawn  |  Hold R = random  |  Esc = stats quit",
        8,
        y,
        scale,
        hint,
    )?;
    y += line;
    let count_line = format!(
        "Vehicles: {}  (must focus game window for keys)",
        vehicle_count
    );
    simple_text::draw_text_utf8(canvas, &BASIC_FONTS, &count_line, 8, y, scale, hint)?;

    canvas.present();
    Ok(())
}

fn show_stats_window(sdl: &sdl2::Sdl, stats: &stats::StatsCollector) -> Result<(), String> {
    let video = sdl.video()?;
    let window = video
        .window("Simulation Statistics", 520, 300)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl.event_pump()?;
    let line_h = 8 * 2 + 8; // scale 2 + spacing
    let lines = [
        format!("Max vehicles passed: {}", stats.vehicles_passed),
        format!("Max velocity: {:.1}", stats.max_velocity),
        format!("Min velocity: {:.1}", stats.min_velocity_display()),
        format!(
            "Max time through intersection: {:.2} s",
            stats.max_time_through_intersection
        ),
        format!(
            "Min time through intersection: {:.2} s",
            stats.min_time_display()
        ),
        format!("Close calls: {}", stats.close_calls),
        String::from("Press Esc to close"),
    ];
    'stats: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'stats;
            }
            if let Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } = event
            {
                break 'stats;
            }
        }

        canvas.set_draw_color(Color::RGB(30, 32, 38));
        canvas.clear();

        let white = Color::RGB(230, 230, 235);
        for (i, line) in lines.iter().enumerate() {
            simple_text::draw_text_utf8(
                &mut canvas,
                &BASIC_FONTS,
                line,
                20,
                20 + (i as i32) * line_h,
                2,
                white,
            )?;
        }

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}
