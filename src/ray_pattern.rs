use rand::prelude::*;
use crate::sorter::{SortVisualizer, SortAlgorithm, SortState};
use crate::render::{draw_line, draw_filled_circle, draw_shadow_glow};
use crate::integration::{
    set_monitor_dimensions as set_monitor_dims, 
    get_monitor_dimensions, 
    initialize_audio_integration, 
    update_and_draw_audio,
    initialize_text_renderer,
    update_and_draw_text
};

const RAY_COUNT: usize = 60;

static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;
static mut YELLOW_POS: Option<(f32, f32)> = None;
static mut GREEN_POS: Option<(f32, f32)> = None;
static mut YELLOW_VEL: Option<(f32, f32)> = None;
static mut GREEN_VEL: Option<(f32, f32)> = None;
static mut LAST_TIME: Option<f32> = None;

pub fn set_monitor_dimensions(monitor: &winit::monitor::MonitorHandle) {
    set_monitor_dims(monitor);
}

pub fn draw_frame(frame: &mut [u8], width: u32, height: u32, time: f32, x_offset: usize, buffer_width: u32) {
    let (scale_x, scale_y) = get_scale_factors(width, height);
    
    initialize_systems();
    update_physics(width, height, time, scale_x, scale_y);
    clear_frame(frame);
    draw_balls_and_rays(frame, width, height, time, scale_x, scale_y, x_offset, buffer_width);
    draw_sorter_visualizations(frame, width, height, time, scale_x, scale_y, x_offset, buffer_width);
    update_and_draw_audio(frame, width, height, time, x_offset, buffer_width);
    update_and_draw_text(frame, width, height, time, x_offset, buffer_width);
}

fn get_scale_factors(width: u32, height: u32) -> (f32, f32) {
    let (monitor_width, monitor_height) = get_monitor_dimensions();
    match (monitor_width, monitor_height) {
        (Some(m_width), Some(m_height)) => {
            let base_width = 1920.0;
            let base_height = 1080.0;
            (m_width as f32 / base_width, m_height as f32 / base_height)
        },
        _ => (1.0, 1.0)
    }
}

fn initialize_systems() {
    initialize_audio_integration();
    initialize_text_renderer();
    unsafe {
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bubble));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bogo));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
    }
}

fn update_physics(width: u32, height: u32, time: f32, scale_x: f32, scale_y: f32) {
    unsafe {
        if YELLOW_POS.is_none() {
            let quarter_width = width as f32 / 4.0;
            let quarter_height = height as f32 / 4.0;
            let vel_scale = (scale_x + scale_y) / 2.0;
            let base_vel_x = 1.0 * vel_scale;
            let base_vel_y = 0.5 * vel_scale;
            YELLOW_POS = Some((quarter_width * 1.5, quarter_height * 1.5));
            YELLOW_VEL = Some((base_vel_x, base_vel_y));
            GREEN_POS = Some((width as f32 - quarter_width * 1.5, height as f32 - quarter_height * 1.5));
            GREEN_VEL = Some((-base_vel_x, -base_vel_y));
        }

        let dt = calculate_delta_time(time);
        update_ball_position(&mut YELLOW_POS, &mut YELLOW_VEL, width, height, dt, scale_x, scale_y);
        update_ball_position(&mut GREEN_POS, &mut GREEN_VEL, width, height, dt, scale_x, scale_y);
        handle_ball_collision();
    }
}

fn calculate_delta_time(time: f32) -> f32 {
    unsafe {
        let dt = if let Some(last) = LAST_TIME {
            let delta = time - last;
            if delta > 0.1 { 0.1 } else { delta }
        } else {
            0.016
        };
        LAST_TIME = Some(time);
        dt
    }
}

fn update_ball_position(pos: &mut Option<(f32, f32)>, vel: &mut Option<(f32, f32)>, 
                       width: u32, height: u32, dt: f32, scale_x: f32, scale_y: f32) {
    if let (Some(pos), Some(vel)) = (pos.as_mut(), vel.as_mut()) {
        let speed_scale = (scale_x + scale_y) / 2.0;
        let base_speed = 50.0 * speed_scale;
        pos.0 += vel.0 * base_speed * dt;
        pos.1 += vel.1 * base_speed * dt;
        
        if pos.0 < 20.0 {
            pos.0 = 20.0;
            vel.0 = vel.0.abs();
        } else if pos.0 > width as f32 - 20.0 {
            pos.0 = width as f32 - 20.0;
            vel.0 = -vel.0.abs();
        }
        if pos.1 < 20.0 {
            pos.1 = 20.0;
            vel.1 = vel.1.abs();
        } else if pos.1 > height as f32 - 20.0 {
            pos.1 = height as f32 - 20.0;
            vel.1 = -vel.1.abs();
        }
    }
}

fn handle_ball_collision() {
    unsafe {
        if let (Some(yellow_pos), Some(green_pos), Some(green_vel)) = 
            (YELLOW_POS.as_mut(), GREEN_POS.as_mut(), GREEN_VEL.as_mut()) {
            let dx = green_pos.0 - yellow_pos.0;
            let dy = green_pos.1 - yellow_pos.1;
            let dist_sq = dx * dx + dy * dy;
            let min_dist = 30.0;
            
            if dist_sq < min_dist * min_dist {
                let dist = dist_sq.sqrt();
                let nx = dx / dist;
                let ny = dy / dist;
                green_pos.0 = yellow_pos.0 + nx * min_dist;
                green_pos.1 = yellow_pos.1 + ny * min_dist;
                
                if let Some(yellow_vel) = YELLOW_VEL.as_mut() {
                    let dot_prod = green_vel.0 * nx + green_vel.1 * ny - yellow_vel.0 * nx - yellow_vel.1 * ny;
                    green_vel.0 -= dot_prod * nx;
                    green_vel.1 -= dot_prod * ny;
                    yellow_vel.0 += dot_prod * nx;
                    yellow_vel.1 += dot_prod * ny;
                }
            }
        }
    }
}

fn clear_frame(frame: &mut [u8]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 5;
        pixel[1] = 5;
        pixel[2] = 10;
        pixel[3] = 255;
    }
}

fn draw_balls_and_rays(frame: &mut [u8], width: u32, height: u32, time: f32, 
                      scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32) {
    unsafe {
        if let Some(yellow_pos) = YELLOW_POS {
            draw_ball_with_effects(frame, width, height, yellow_pos, [255, 255, 0, 255], 
                                 [255, 255, 150, 255], time, scale_x, scale_y, x_offset, buffer_width);
        }
        if let Some(green_pos) = GREEN_POS {
            draw_ball_with_effects(frame, width, height, green_pos, [0, 255, 0, 255], 
                                 [150, 255, 150, 255], time + 0.5, scale_x, scale_y, x_offset, buffer_width);
        }
    }
}

fn draw_ball_with_effects(frame: &mut [u8], width: u32, height: u32, pos: (f32, f32), 
                         ball_color: [u8; 4], ray_color: [u8; 4], time: f32, 
                         scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32) {
    draw_rays(frame, width, height, pos.0 as i32, pos.1 as i32, 
              width as i32 / 2, height as i32 / 2, width as i32 / 2 - 20, 
              &ray_color, RAY_COUNT, time, x_offset, buffer_width);
    
    let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
    draw_filled_circle(frame, width, height, pos.0 as i32, pos.1 as i32, 
                      ball_radius, &ball_color, x_offset, buffer_width);
    
    let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
    let glow_color = [ball_color[0], ball_color[1], (ball_color[2] as f32 * 0.4) as u8, 100];
    draw_shadow_glow(frame, width, height, pos.0 as i32, pos.1 as i32, 
                    glow_radius, &glow_color, x_offset, buffer_width);
}

fn draw_sorter_visualizations(frame: &mut [u8], width: u32, height: u32, time: f32, 
                            scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32) {
    let scale_factor = (scale_x + scale_y) / 2.0;
    let border_thickness = (height as f32 * 0.05 * scale_factor) as usize;
    let side_width = (width as f32 * 0.05 * scale_factor) as usize;
    
    unsafe {
        update_and_draw_sorter(&mut TOP_SORTER, frame, 0, 0, width as usize, border_thickness, 
                              true, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut BOTTOM_SORTER, frame, 0, height as usize - border_thickness, 
                              width as usize, border_thickness, true, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut LEFT_SORTER, frame, 0, border_thickness, side_width, 
                              height as usize - border_thickness * 2, false, time, x_offset, buffer_width);
        update_and_draw_sorter(&mut RIGHT_SORTER, frame, width as usize - side_width, border_thickness, 
                              side_width, height as usize - border_thickness * 2, false, time, x_offset, buffer_width);
    }
}

fn update_and_draw_sorter(sorter: &mut Option<SortVisualizer>, frame: &mut [u8], 
                         x: usize, y: usize, width: usize, height: usize, 
                         horizontal: bool, time: f32, x_offset: usize, buffer_width: u32) {
    if let Some(sorter) = sorter {
        sorter.update();
        if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
            sorter.restart();
        }
        sorter.draw(frame, x, y, width, height, horizontal, x_offset, buffer_width as u32);
    }
}

pub fn apply_force_yellow(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = YELLOW_VEL {
            YELLOW_VEL = Some((vel.0 + force_x, vel.1 + force_y));
        }
    }
}

pub fn apply_force_green(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = GREEN_VEL {
            GREEN_VEL = Some((vel.0 + force_x, vel.1 + force_y));
        }
    }
}

pub fn teleport_yellow(x: f32, y: f32) { 
    unsafe { YELLOW_POS = Some((x, y)); } 
}

pub fn teleport_green(x: f32, y: f32) { 
    unsafe { GREEN_POS = Some((x, y)); } 
}

pub fn restart_sorters() {
    unsafe {
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.restart();
        }
    }
}

fn draw_rays(frame: &mut [u8], width: u32, height: u32, 
            source_x: i32, source_y: i32, 
            center_x: i32, center_y: i32,
            radius: i32, color: &[u8; 4], 
            count: usize, time: f32, 
            x_offset: usize, buffer_width: u32) {
    let (other_x, other_y, other_radius) = unsafe {
        if source_x == YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32 && 
           source_y == YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32 {
            (GREEN_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             GREEN_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10)
        } else {
            (YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10)
        }
    };
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
                draw_line(frame, width, height, source_x, source_y, intersect_x, intersect_y, 
                         color, x_offset, buffer_width);
                let shadow_length = radius as f32 * 1.2;
                let shadow_end_x = (intersect_x as f32 + ray_dir_x * shadow_length) as i32;
                let shadow_end_y = (intersect_y as f32 + ray_dir_y * shadow_length) as i32;
                shadow_rays.push(((intersect_x, intersect_y), (shadow_end_x, shadow_end_y)));
            } else {
                draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                         color, x_offset, buffer_width);
            }
        } else {
            draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                     color, x_offset, buffer_width);
        }
    }
    let shadow_color = [
        (color[0] as f32 * 0.2) as u8,
        (color[1] as f32 * 0.2) as u8,
        (color[2] as f32 * 0.2) as u8,
        128,
    ];
    for shadow in shadow_rays {
        draw_line(frame, width, height, 
                 shadow.0.0, shadow.0.1, 
                 shadow.1.0, shadow.1.1, 
                 &shadow_color, x_offset, buffer_width);
    }
}
