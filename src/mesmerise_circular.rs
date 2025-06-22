use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    keyboard::KeyCode,
    event_loop::{EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use std::time::{Duration, Instant};

/// Canvas size (split into three vertical regions)
pub const WIDTH: u32 = 900;
pub const HEIGHT: u32 = 300;

pub fn draw_frame(frame: &mut [u8], t: f32) {
    let w = WIDTH as usize;
    let h = HEIGHT as usize;
    let region_width = w / 3;

    for px in frame.chunks_exact_mut(4) {
        px[0] = 0;
        px[1] = 0;
        px[2] = 0;
        px[3] = 255;
    }

    draw_concentric_region(frame, 0, region_width, w, h, t);
    draw_orbit_region(frame, region_width, region_width, w, h, t);
    draw_beams_region(frame, 2 * region_width, region_width, w, h, t);
}

/// Region 0: concentric, rainbow‐colored circles
pub fn draw_concentric_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let radius = (region_w.min(h) as f32) * 0.45;

    for py in 0..h {
        for px in 0..region_w {
            let dx = px as f32 - region_w as f32 / 2.0;
            let dy = py as f32 - h as f32 / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let r = (angle * 6.0 + t).sin() * 0.5 + 0.5;
            let g = (angle * 4.0 + t * 1.2).sin() * 0.5 + 0.5;
            let b = (angle * 8.0 + t * 0.8).sin() * 0.5 + 0.5;

            let alpha = if dist <= radius {
                1.0
            } else {
                1.0 - ((dist - radius) / (radius * 0.4)).clamp(0.0, 1.0)
            };

            let x = x_off + px;
            let idx = 4 * (py * w + x);
            frame[idx]     = (255.0 * r * alpha) as u8;
            frame[idx + 1] = (255.0 * g * alpha) as u8;
            frame[idx + 2] = (255.0 * b * alpha) as u8;
            frame[idx + 3] = 255;
        }
    }
}

/// Region 1: a few dots orbiting in a circle
pub fn draw_orbit_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let cx = x_off + region_w / 2;
    let cy = h / 2;
    let radius = (region_w.min(h) as f32) * 0.4;
    let count = 5;
    let size = 3;

    for i in 0..count {
        let phase = t * 1.2 + (i as f32 / count as f32) * std::f32::consts::TAU;
        let sx = (cx as f32 + radius * phase.cos()) as i32;
        let sy = (cy as f32 + radius * phase.sin()) as i32;

        for oy in -size..=size {
            for ox in -size..=size {
                let px = sx + ox;
                let py = sy + oy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let idx = 4 * (py as usize * w + px as usize);
                    frame[idx]     = 50;
                    frame[idx + 1] = 200;
                    frame[idx + 2] = 50;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
}

/// Region 2: radial white beams spinning slowly
pub fn draw_beams_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let cx = x_off + region_w / 2;
    let cy = h / 2;
    let beams = 16;
    let length = (region_w.min(h) as f32) * 0.45;

    for k in 0..beams {
        let angle = k as f32 * (std::f32::consts::TAU / beams as f32) + t * 0.2;
        let dx = angle.cos();
        let dy = angle.sin();
        let mut x = cx as f32;
        let mut y = cy as f32;

        for _ in 0..(length as usize) {
            let ix = x as i32;
            let iy = y as i32;
            if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                let idx = 4 * (iy as usize * w + ix as usize);
                frame[idx]     = 255;
                frame[idx + 1] = 255;
                frame[idx + 2] = 255;
                frame[idx + 3] = 255;
            }
            x += dx;
            y += dy;
        }
    }
}
