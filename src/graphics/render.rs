pub trait Drawer {
    fn draw_line(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    );

    fn draw_filled_circle(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        center_x: i32,
        center_y: i32,
        radius: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    );

    fn draw_shadow_glow(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        center_x: i32,
        center_y: i32,
        radius: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    );
}

pub struct Renderer;

impl Drawer for Renderer {
    fn draw_line(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    ) {
        draw_line_internal(
            frame,
            width,
            height,
            x0,
            y0,
            x1,
            y1,
            color,
            x_offset,
            buffer_width,
        );
    }

    fn draw_filled_circle(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        center_x: i32,
        center_y: i32,
        radius: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    ) {
        draw_filled_circle_internal(
            frame,
            width,
            height,
            center_x,
            center_y,
            radius,
            color,
            x_offset,
            buffer_width,
        );
    }

    fn draw_shadow_glow(
        &self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        center_x: i32,
        center_y: i32,
        radius: i32,
        color: &[u8; 4],
        x_offset: usize,
        buffer_width: u32,
    ) {
        draw_shadow_glow_internal(
            frame,
            width,
            height,
            center_x,
            center_y,
            radius,
            color,
            x_offset,
            buffer_width,
        );
    }
}

fn draw_line_internal(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel(frame, width, height, x0, y0, color, x_offset, buffer_width);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            err += dx;
            y0 += sy;
        }
    }
}

pub fn draw_line(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    draw_line_internal(
        frame,
        width,
        height,
        x0,
        y0,
        x1,
        y1,
        color,
        x_offset,
        buffer_width,
    );
}

fn draw_filled_circle_internal(
    frame: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y <= radius * radius {
                put_pixel(
                    frame,
                    width,
                    height,
                    center_x + x,
                    center_y + y,
                    color,
                    x_offset,
                    buffer_width,
                );
            }
        }
    }
}

pub fn draw_filled_circle(
    frame: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    draw_filled_circle_internal(
        frame,
        width,
        height,
        center_x,
        center_y,
        radius,
        color,
        x_offset,
        buffer_width,
    );
}

fn draw_shadow_glow_internal(
    frame: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            let dist_sq = x * x + y * y;
            if dist_sq <= radius * radius {
                let pixel_x = center_x + x;
                let pixel_y = center_y + y;

                if pixel_x >= 0 && pixel_x < width as i32 && pixel_y >= 0 && pixel_y < height as i32
                {
                    let idx = 4
                        * ((pixel_y as usize * buffer_width as usize)
                            + pixel_x as usize
                            + x_offset);

                    if idx + 3 < frame.len() {
                        let distance = (dist_sq as f32).sqrt();
                        let alpha_factor = 1.0 - (distance / radius as f32);
                        let alpha = (color[3] as f32 * alpha_factor) as u8;

                        let shadow_color = [
                            (color[0] as f32 * alpha_factor) as u8,
                            (color[1] as f32 * alpha_factor) as u8,
                            (color[2] as f32 * alpha_factor) as u8,
                            alpha,
                        ];

                        let existing_r = frame[idx];
                        let existing_g = frame[idx + 1];
                        let existing_b = frame[idx + 2];

                        frame[idx] = existing_r.saturating_add(shadow_color[0]);
                        frame[idx + 1] = existing_g.saturating_add(shadow_color[1]);
                        frame[idx + 2] = existing_b.saturating_add(shadow_color[2]);
                        frame[idx + 3] = 255;
                    }
                }
            }
        }
    }
}

pub fn draw_shadow_glow(
    frame: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    draw_shadow_glow_internal(
        frame,
        width,
        height,
        center_x,
        center_y,
        radius,
        color,
        x_offset,
        buffer_width,
    );
}

fn put_pixel(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: &[u8; 4],
    x_offset: usize,
    buffer_width: u32,
) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        let idx = 4 * ((y as usize * buffer_width as usize) + x as usize + x_offset);

        if idx + 3 < frame.len() {
            let alpha = color[3] as f32 / 255.0;
            frame[idx] = (frame[idx] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8;
            frame[idx + 1] =
                (frame[idx + 1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8;
            frame[idx + 2] =
                (frame[idx + 2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8;
            frame[idx + 3] = 255;
        }
    }
}

pub fn draw_rays_from_ball(
    frame: &mut [u8],
    width: u32,
    height: u32,
    pos: (f32, f32),
    ray_color: [u8; 4],
    time: f32,
    x_offset: usize,
    buffer_width: u32,
    other_pos: (f32, f32),
) {
    let source_x = pos.0 as i32;
    let source_y = pos.1 as i32;
    let center_x = width as i32 / 2;
    let center_y = height as i32 / 2;
    let radius = width as i32 / 2 - 20;
    let count = 60;

    let other_x = other_pos.0 as i32;
    let other_y = other_pos.1 as i32;
    let other_radius = 10;

    let mut shadow_rays: Vec<((i32, i32), (i32, i32))> = Vec::new();

    for i in 0..count {
        let base_angle = (i as f32 / count as f32) * 2.0 * std::f32::consts::PI;
        let angle = base_angle + (time * 0.2).sin() * 0.05;
        let end_x = center_x as f32 + angle.cos() * radius as f32;
        let end_y = center_y as f32 + angle.sin() * radius as f32;

        let ray_dir_x = end_x as f32 - source_x as f32;
        let ray_dir_y = end_y as f32 - source_y as f32;
        let ray_length = (ray_dir_x * ray_dir_x + ray_dir_y * ray_dir_y).sqrt();
        let ray_dir_x = ray_dir_x / ray_length;
        let ray_dir_y = ray_dir_y / ray_length;

        let oc_x = source_x as f32 - other_x as f32;
        let oc_y = source_y as f32 - other_y as f32;
        let a = 1.0;
        let b = 2.0 * (ray_dir_x * oc_x + ray_dir_y * oc_y);
        let c = (oc_x * oc_x + oc_y * oc_y) - (other_radius * other_radius) as f32;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant >= 0.0 {
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
            if (t1 > 0.0 && t1 < ray_length) || (t2 > 0.0 && t2 < ray_length) {
                let t = t1.max(0.0);
                let intersect_x = (source_x as f32 + ray_dir_x * t) as i32;
                let intersect_y = (source_y as f32 + ray_dir_y * t) as i32;
                draw_line_internal(
                    frame,
                    width,
                    height,
                    source_x,
                    source_y,
                    intersect_x,
                    intersect_y,
                    &ray_color,
                    x_offset,
                    buffer_width,
                );

                let shadow_length = radius as f32 * 1.2;
                let shadow_end_x = (intersect_x as f32 + ray_dir_x * shadow_length) as i32;
                let shadow_end_y = (intersect_y as f32 + ray_dir_y * shadow_length) as i32;
                shadow_rays.push(((intersect_x, intersect_y), (shadow_end_x, shadow_end_y)));
            } else {
                draw_line_internal(
                    frame,
                    width,
                    height,
                    source_x,
                    source_y,
                    end_x as i32,
                    end_y as i32,
                    &ray_color,
                    x_offset,
                    buffer_width,
                );
            }
        } else {
            draw_line_internal(
                frame,
                width,
                height,
                source_x,
                source_y,
                end_x as i32,
                end_y as i32,
                &ray_color,
                x_offset,
                buffer_width,
            );
        }
    }

    let shadow_color = [
        (ray_color[0] as f32 * 0.2) as u8,
        (ray_color[1] as f32 * 0.2) as u8,
        (ray_color[2] as f32 * 0.2) as u8,
        128,
    ];

    for shadow in shadow_rays {
        draw_line_internal(
            frame,
            width,
            height,
            shadow.0 .0,
            shadow.0 .1,
            shadow.1 .0,
            shadow.1 .1,
            &shadow_color,
            x_offset,
            buffer_width,
        );
    }
}

pub fn clear_frame(frame: &mut [u8]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 5;
        pixel[1] = 5;
        pixel[2] = 10;
        pixel[3] = 255;
    }
}
