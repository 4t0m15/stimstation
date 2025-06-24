use crate::render::{draw_filled_circle, draw_shadow_glow};

static mut YELLOW_POS: Option<(f32, f32)> = None;
static mut GREEN_POS: Option<(f32, f32)> = None;
static mut YELLOW_VEL: Option<(f32, f32)> = None;
static mut GREEN_VEL: Option<(f32, f32)> = None;
static mut LAST_TIME: Option<f32> = None;

pub fn initialize_balls(width: u32, height: u32, scale_x: f32, scale_y: f32) {
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
    }
}

pub fn get_ball_positions() -> (Option<(f32, f32)>, Option<(f32, f32)>) {
    unsafe { (YELLOW_POS, GREEN_POS) }
}

pub fn update_physics(width: u32, height: u32, time: f32, scale_x: f32, scale_y: f32) {
    let dt = calculate_delta_time(time);
    unsafe {
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

pub fn draw_balls_with_effects(frame: &mut [u8], width: u32, height: u32, time: f32, 
                              scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32,
                              draw_rays_fn: impl Fn(&mut [u8], u32, u32, (f32, f32), [u8; 4], f32, usize, u32)) {
    unsafe {
        if let Some(yellow_pos) = YELLOW_POS {
            draw_ball_with_effects(frame, width, height, yellow_pos, [255, 255, 0, 255], 
                                 [255, 255, 150, 255], time, scale_x, scale_y, x_offset, buffer_width, &draw_rays_fn);
        }
        if let Some(green_pos) = GREEN_POS {
            draw_ball_with_effects(frame, width, height, green_pos, [0, 255, 0, 255], 
                                 [150, 255, 150, 255], time + 0.5, scale_x, scale_y, x_offset, buffer_width, &draw_rays_fn);
        }
    }
}

fn draw_ball_with_effects(frame: &mut [u8], width: u32, height: u32, pos: (f32, f32), 
                         ball_color: [u8; 4], ray_color: [u8; 4], time: f32, 
                         scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32,
                         draw_rays_fn: &impl Fn(&mut [u8], u32, u32, (f32, f32), [u8; 4], f32, usize, u32)) {
    draw_rays_fn(frame, width, height, pos, ray_color, time, x_offset, buffer_width);
    
    let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
    draw_filled_circle(frame, width, height, pos.0 as i32, pos.1 as i32, 
                      ball_radius, &ball_color, x_offset, buffer_width);
    
    let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
    let glow_color = [ball_color[0], ball_color[1], (ball_color[2] as f32 * 0.4) as u8, 100];
    draw_shadow_glow(frame, width, height, pos.0 as i32, pos.1 as i32, 
                    glow_radius, &glow_color, x_offset, buffer_width);
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
