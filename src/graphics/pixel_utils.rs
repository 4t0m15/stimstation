use crate::core::types::{HEIGHT, WIDTH};
pub fn set_pixel_safe(frame: &mut [u8], x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        let idx = 4 * (y as usize * width as usize + x as usize);
        if idx + 3 < frame.len() {
            frame[idx] = color[0];
            frame[idx + 1] = color[1];
            frame[idx + 2] = color[2];
            frame[idx + 3] = color[3];
        }
    }
}
pub fn blend_pixel_safe(
    frame: &mut [u8],
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: [u8; 4],
    intensity: f32,
) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        let idx = 4 * (y as usize * width as usize + x as usize);
        if idx + 3 < frame.len() {
            let r = (intensity * color[0] as f32) as u16;
            let g = (intensity * color[1] as f32) as u16;
            let b = (intensity * color[2] as f32) as u16;
            frame[idx] = (frame[idx] as u16 + r).min(255) as u8;
            frame[idx + 1] = (frame[idx + 1] as u16 + g).min(255) as u8;
            frame[idx + 2] = (frame[idx + 2] as u16 + b).min(255) as u8;
            frame[idx + 3] = color[3];
        }
    }
}

pub fn draw_rectangle_safe(
    frame: &mut [u8],
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: [u8; 4],
    buffer_width: u32,
    buffer_height: u32,
) {
    let x_start = x.max(0) as u32;
    let y_start = y.max(0) as u32;
    let x_end = (x + width as i32).min(buffer_width as i32) as u32;
    let y_end = (y + height as i32).min(buffer_height as i32) as u32;

    let alpha = color[3] as f32 / 255.0;
    let src_r = color[0] as f32;
    let src_g = color[1] as f32;
    let src_b = color[2] as f32;

    for py in y_start..y_end {
        for px in x_start..x_end {
            let idx = 4 * (py as usize * buffer_width as usize + px as usize);
            if idx + 3 < frame.len() {
                let dst_r = frame[idx] as f32;
                let dst_g = frame[idx + 1] as f32;
                let dst_b = frame[idx + 2] as f32;

                frame[idx] = ((src_r * alpha) + (dst_r * (1.0 - alpha))) as u8;
                frame[idx + 1] = ((src_g * alpha) + (dst_g * (1.0 - alpha))) as u8;
                frame[idx + 2] = ((src_b * alpha) + (dst_b * (1.0 - alpha))) as u8;
            }
        }
    }
}

pub fn draw_line(frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], width: i32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    let glow_radius = width * 3;
    let height = frame.len() / (4 * WIDTH as usize);
    if (x0 < 0 && x1 < 0)
        || (x0 >= WIDTH as i32 && x1 >= WIDTH as i32)
        || (y0 < 0 && y1 < 0)
        || (y0 >= height as i32 && y1 >= height as i32)
    {
        return;
    }
    while x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
        for w_y in -glow_radius..=glow_radius {
            for w_x in -glow_radius..=glow_radius {
                let distance_squared = w_x * w_x + w_y * w_y;
                let distance = (distance_squared as f32).sqrt();
                if distance > glow_radius as f32 {
                    continue;
                }
                let intensity = if distance <= width as f32 {
                    1.0
                } else {
                    let falloff =
                        1.0 - (distance - width as f32) / (glow_radius as f32 - width as f32);
                    falloff * falloff
                };
                blend_pixel_safe(
                    frame,
                    x + w_x,
                    y + w_y,
                    WIDTH,
                    HEIGHT as u32,
                    color,
                    intensity,
                );
            }
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
pub fn draw_point(frame: &mut [u8], x: i32, y: i32, color: [u8; 4], size: i32) {
    let glow_radius = size * 2;
    let _height = frame.len() / (4 * WIDTH as usize);
    if x + glow_radius < 0
        || x - glow_radius >= WIDTH as i32
        || y + glow_radius < 0
        || y - glow_radius >= HEIGHT as i32
    {
        return;
    }
    for w_y in -glow_radius..=glow_radius {
        for w_x in -glow_radius..=glow_radius {
            let distance_squared = w_x * w_x + w_y * w_y;
            let distance = (distance_squared as f32).sqrt();
            if distance > glow_radius as f32 {
                continue;
            }
            let intensity = if distance <= size as f32 {
                1.0
            } else {
                let falloff = 1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32);
                falloff * falloff
            };
            let alpha_factor = color[3] as f32 / 255.0;
            let r = (intensity * color[0] as f32 * alpha_factor) as u8;
            let g = (intensity * color[1] as f32 * alpha_factor) as u8;
            let b = (intensity * color[2] as f32 * alpha_factor) as u8;
            blend_pixel_safe(
                frame,
                x + w_x,
                y + w_y,
                WIDTH,
                HEIGHT as u32,
                [r, g, b, color[3]],
                1.0,
            );
        }
    }
}
pub fn draw_circle(frame: &mut [u8], x: i32, y: i32, radius: i32, color: [u8; 4], width: u32) {
    let height = frame.len() / (4 * width as usize);
    if x + radius < 0 || x - radius >= width as i32 || y + radius < 0 || y - radius >= height as i32
    {
        return;
    }
    let radius_sq = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius_sq {
                set_pixel_safe(frame, x + dx, y + dy, width, height as u32, color);
            }
        }
    }
}
pub fn draw_extra_bright_particle(
    frame: &mut [u8],
    x: i32,
    y: i32,
    size: i32,
    color: [u8; 4],
    width: u32,
) {
    let glow_radius = size * 3;
    let height = frame.len() / (4 * width as usize);
    if x + glow_radius < 0
        || x - glow_radius >= width as i32
        || y + glow_radius < 0
        || y - glow_radius >= height as i32
    {
        return;
    }
    for dy in -glow_radius..=glow_radius {
        for dx in -glow_radius..=glow_radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > glow_radius * glow_radius {
                continue;
            }
            let distance = (dist_sq as f32).sqrt();
            let intensity = if distance <= size as f32 {
                2.0
            } else if distance <= glow_radius as f32 {
                1.5 * (1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32))
            } else {
                0.0
            };
            let alpha_factor = color[3] as f32 / 255.0;
            let r = (intensity * color[0] as f32 * alpha_factor * 3.0).min(255.0) as u8;
            let g = (intensity * color[1] as f32 * alpha_factor * 3.0).min(255.0) as u8;
            let b = (intensity * color[2] as f32 * alpha_factor * 3.0).min(255.0) as u8;
            blend_pixel_safe(
                frame,
                x + dx,
                y + dy,
                width,
                height as u32,
                [r, g, b, 255],
                1.0,
            );
        }
    }
}
pub fn draw_huge_text(frame: &mut [u8], text: &str, x: i32, y: i32, color: [u8; 4], width: u32) {
    let char_width = 30;
    let char_height = 50;
    let stroke_width = 4;
    let height = frame.len() / (4 * width as usize);
    if y + char_height < 0 || y >= height as i32 {
        return;
    }
    for (i, _c) in text.chars().enumerate() {
        let cx = x + (i as i32 * char_width);
        if cx + char_width < 0 || cx >= width as i32 {
            continue;
        }
        for dy in 0..char_height {
            for dx in 0..char_width {
                let is_border = dx < stroke_width
                    || dx >= char_width - stroke_width
                    || dy < stroke_width
                    || dy >= char_height - stroke_width;
                if is_border {
                    set_pixel_safe(frame, cx + dx, y + dy, width, height as u32, color);
                }
            }
        }
    }
}
pub fn draw_border(
    frame: &mut [u8],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: [u8; 4],
    stride: u32,
) {
    let border_width = 3;
    for dy in 0..border_width {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;
            let idx = 4 * (py as usize * stride as usize + px as usize);
            if idx + 3 < frame.len() {
                frame[idx] = color[0];
                frame[idx + 1] = color[1];
                frame[idx + 2] = color[2];
                frame[idx + 3] = color[3];
            }
        }
    }
    for dy in 0..border_width {
        for dx in 0..width {
            let px = x + dx;
            let py = y + height - 1 - dy;
            let idx = 4 * (py as usize * stride as usize + px as usize);
            if idx + 3 < frame.len() {
                frame[idx] = color[0];
                frame[idx + 1] = color[1];
                frame[idx + 2] = color[2];
                frame[idx + 3] = color[3];
            }
        }
    }
    for dx in 0..border_width {
        for dy in 0..height {
            let px = x + dx;
            let py = y + dy;
            let idx = 4 * (py as usize * stride as usize + px as usize);
            if idx + 3 < frame.len() {
                frame[idx] = color[0];
                frame[idx + 1] = color[1];
                frame[idx + 2] = color[2];
                frame[idx + 3] = color[3];
            }
        }
    }
    for dx in 0..border_width {
        for dy in 0..height {
            let px = x + width - 1 - dx;
            let py = y + dy;
            let idx = 4 * (py as usize * stride as usize + px as usize);
            if idx + 3 < frame.len() {
                frame[idx] = color[0];
                frame[idx + 1] = color[1];
                frame[idx + 2] = color[2];
                frame[idx + 3] = color[3];
            }
        }
    }
}
pub fn draw_segment(
    frame: &mut [u8],
    x: i32,
    y: i32,
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    e: bool,
    f: bool,
    g: bool,
    color: [u8; 4],
    width: u32,
) {
    let height = frame.len() / (4 * width as usize);
    let thickness = 2;
    if a {
        for dy in 0..thickness {
            for dx in 0..6 {
                set_pixel_safe(frame, x + 1 + dx, y + dy, width, height as u32, color);
            }
        }
    }
    if b {
        for dy in 0..7 {
            for dx in 0..thickness {
                set_pixel_safe(frame, x + 6 - dx, y + 1 + dy, width, height as u32, color);
            }
        }
    }
    if c {
        for dy in 0..7 {
            for dx in 0..thickness {
                set_pixel_safe(frame, x + 6 - dx, y + 8 + dy, width, height as u32, color);
            }
        }
    }
    if d {
        for dy in 0..thickness {
            for dx in 0..6 {
                set_pixel_safe(frame, x + 1 + dx, y + 14 - dy, width, height as u32, color);
            }
        }
    }
    if e {
        for dy in 0..7 {
            for dx in 0..thickness {
                set_pixel_safe(frame, x + dx, y + 8 + dy, width, height as u32, color);
            }
        }
    }
    if f {
        for dy in 0..7 {
            for dx in 0..thickness {
                set_pixel_safe(frame, x + dx, y + 1 + dy, width, height as u32, color);
            }
        }
    }
    if g {
        for dy in 0..thickness {
            for dx in 0..6 {
                set_pixel_safe(frame, x + 1 + dx, y + 7 + dy, width, height as u32, color);
            }
        }
    }
}
pub fn draw_triangle_filled(
    frame: &mut [u8],
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    x3: i32,
    y3: i32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    let mut vertices = [(x1, y1), (x2, y2), (x3, y3)];
    vertices.sort_by_key(|&(_, y)| y);
    let [(x1, y1), (x2, y2), (x3, y3)] = vertices;
    if y2 > y1 {
        let slope1 = (x2 - x1) as f32 / (y2 - y1) as f32;
        let slope2 = (x3 - x1) as f32 / (y3 - y1) as f32;
        for y in y1..=y2 {
            let dy = y - y1;
            let start_x = (x1 as f32 + slope1 * dy as f32) as i32;
            let end_x = (x1 as f32 + slope2 * dy as f32) as i32;
            for x in std::cmp::min(start_x, end_x)..=std::cmp::max(start_x, end_x) {
                set_pixel_safe(frame, x, y, width, height, color);
            }
        }
    }
    if y3 > y2 {
        let slope1 = (x3 - x2) as f32 / (y3 - y2) as f32;
        let slope2 = (x3 - x1) as f32 / (y3 - y1) as f32;
        for y in y2 + 1..=y3 {
            let dy1 = y - y2;
            let dy2 = y - y1;
            let start_x = (x2 as f32 + slope1 * dy1 as f32) as i32;
            let end_x = (x1 as f32 + slope2 * dy2 as f32) as i32;
            for x in std::cmp::min(start_x, end_x)..=std::cmp::max(start_x, end_x) {
                set_pixel_safe(frame, x, y, width, height, color);
            }
        }
    }
}
