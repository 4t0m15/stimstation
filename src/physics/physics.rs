#![allow(unsafe_op_in_unsafe_fn)]
#![allow(static_mut_refs)]

use crate::audio::audio_handler::get_audio_spectrum;
use crate::graphics::render::draw_filled_circle;

/// Holds the positions and velocities of both balls.
struct BallState {
    yellow_pos: Option<(f32, f32)>,
    green_pos: Option<(f32, f32)>,
    yellow_vel: Option<(f32, f32)>,
    green_vel: Option<(f32, f32)>,
    last_time: Option<f32>,
}

// Single static state object (preferably replaced with a higher-level manager).
static mut BALL_STATE: Option<BallState> = None;

/// Initializes both balls if not already initialized.
pub fn initialize_balls(width: u32, height: u32, scale_x: f32, scale_y: f32) {
    unsafe {
        if BALL_STATE.is_none() {
            BALL_STATE = Some(BallState {
                yellow_pos: None,
                green_pos: None,
                yellow_vel: None,
                green_vel: None,
                last_time: None,
            });
        }
        let state = BALL_STATE.as_mut().unwrap();
        if state.yellow_pos.is_none() {
            let quarter_width = width as f32 / 4.0;
            let quarter_height = height as f32 / 4.0;
            let vel_scale = (scale_x + scale_y) / 2.0;
            let base_vel_x = 1.0 * vel_scale;
            let base_vel_y = 0.5 * vel_scale;
            state.yellow_pos = Some((quarter_width * 1.5, quarter_height * 1.5));
            state.yellow_vel = Some((base_vel_x, base_vel_y));
            state.green_pos = Some((
                width as f32 - quarter_width * 1.5,
                height as f32 - quarter_height * 1.5,
            ));
            state.green_vel = Some((-base_vel_x, -base_vel_y));
        }
    }
}

/// Returns the ball positions for drawing or other logic.
pub fn get_ball_positions() -> (Option<(f32, f32)>, Option<(f32, f32)>) {
    unsafe {
        let state = BALL_STATE.as_ref().unwrap();
        (state.yellow_pos, state.green_pos)
    }
}

/// Main update step for physics; updates positions and checks collisions.
pub fn update_physics(width: u32, height: u32, time: f32, scale_x: f32, scale_y: f32) {
    initialize_balls(width, height, scale_x, scale_y);
    let dt = calculate_delta_time(time);
    unsafe {
        update_ball_position(
            &mut BALL_STATE.as_mut().unwrap().yellow_pos,
            &mut BALL_STATE.as_mut().unwrap().yellow_vel,
            width,
            height,
            dt,
            scale_x,
            scale_y,
        );
        update_ball_position(
            &mut BALL_STATE.as_mut().unwrap().green_pos,
            &mut BALL_STATE.as_mut().unwrap().green_vel,
            width,
            height,
            dt,
            scale_x,
            scale_y,
        );
        handle_ball_collision();
    }
}

fn calculate_delta_time(time: f32) -> f32 {
    unsafe {
        let state = BALL_STATE.as_mut().unwrap();
        let dt = if let Some(last) = state.last_time {
            let delta = time - last;
            if delta > 0.1 {
                0.1
            } else {
                delta
            }
        } else {
            0.016
        };
        state.last_time = Some(time);
        dt
    }
}

fn update_ball_position(
    pos: &mut Option<(f32, f32)>,
    vel: &mut Option<(f32, f32)>,
    width: u32,
    height: u32,
    dt: f32,
    scale_x: f32,
    scale_y: f32,
) {
    if let (Some(pos), Some(vel)) = (pos.as_mut(), vel.as_mut()) {
        let speed_scale = (scale_x + scale_y) / 2.0;
        let base_speed = 50.0 * speed_scale;
        pos.0 += vel.0 * base_speed * dt;
        pos.1 += vel.1 * base_speed * dt;

        if pos.0 < 20.0 {
            pos.0 = 20.0;
            vel.0 = vel.0.abs();
            crate::physics::detect_corner::increment_corner_hit(pos.0, pos.1, width, height);
        } else if pos.0 > width as f32 - 20.0 {
            pos.0 = width as f32 - 20.0;
            vel.0 = -vel.0.abs();
            crate::physics::detect_corner::increment_corner_hit(pos.0, pos.1, width, height);
        }
        if pos.1 < 20.0 {
            pos.1 = 20.0;
            vel.1 = vel.1.abs();
            crate::physics::detect_corner::increment_corner_hit(pos.0, pos.1, width, height);
        } else if pos.1 > height as f32 - 20.0 {
            pos.1 = height as f32 - 20.0;
            vel.1 = -vel.1.abs();
            crate::physics::detect_corner::increment_corner_hit(pos.0, pos.1, width, height);
        }
    }
}

fn handle_ball_collision() {
    unsafe {
        let state = BALL_STATE.as_mut().unwrap();
        if let (Some(yellow_pos), Some(green_pos), Some(yellow_vel), Some(green_vel)) = (
            state.yellow_pos.as_mut(),
            state.green_pos.as_mut(),
            state.yellow_vel.as_mut(),
            state.green_vel.as_mut(),
        ) {
            let dx = green_pos.0 - yellow_pos.0;
            let dy = green_pos.1 - yellow_pos.1;
            let dist_sq = dx * dx + dy * dy;
            let min_dist = 60.0; // Much larger collision distance to ensure detection

            if dist_sq < min_dist * min_dist && dist_sq > 0.0 {
                let dist = dist_sq.sqrt();
                let nx = dx / dist;
                let ny = dy / dist;

                // Separate the balls to prevent overlap
                let overlap = min_dist - dist;
                let separation = overlap * 0.5;
                yellow_pos.0 -= nx * separation;
                yellow_pos.1 -= ny * separation;
                green_pos.0 += nx * separation;
                green_pos.1 += ny * separation;

                // Calculate relative velocity
                let rel_vel_x = green_vel.0 - yellow_vel.0;
                let rel_vel_y = green_vel.1 - yellow_vel.1;

                // Calculate relative velocity along collision normal
                let vel_along_normal = rel_vel_x * nx + rel_vel_y * ny;

                // Don't resolve if velocities are separating
                if vel_along_normal > 0.0 {
                    return;
                }

                // Calculate restitution (bounciness) - make it much more bouncy
                let restitution = 1.2; // More than 1 for super bouncy effect
                let impulse_magnitude = -(1.0 + restitution) * vel_along_normal;

                // Apply impulse to both balls (assuming equal mass) - make it more dramatic
                let impulse_x = impulse_magnitude * nx * 0.5;
                let impulse_y = impulse_magnitude * ny * 0.5;

                // Add some random deflection for more interesting bounces
                let random_factor = (dist_sq.sin() * 0.1) as f32;

                yellow_vel.0 -= impulse_x + random_factor;
                yellow_vel.1 -= impulse_y + random_factor;
                green_vel.0 += impulse_x + random_factor;
                green_vel.1 += impulse_y + random_factor;
            }
        }
    }
}

pub fn draw_balls_with_effects(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    scale_x: f32,
    scale_y: f32,
    x_offset: usize,
    buffer_width: u32,
    draw_rays_fn: impl Fn(&mut [u8], u32, u32, (f32, f32), [u8; 4], f32, usize, u32),
) {
    unsafe {
        let state = BALL_STATE.as_ref().unwrap();
        if let Some(yellow_pos) = state.yellow_pos {
            draw_ball_with_effects(
                frame,
                width,
                height,
                yellow_pos,
                [255, 255, 0, 255],
                [255, 255, 150, 255],
                time,
                scale_x,
                scale_y,
                x_offset,
                buffer_width,
                &draw_rays_fn,
                true,
            );
        }
        if let Some(green_pos) = state.green_pos {
            draw_ball_with_effects(
                frame,
                width,
                height,
                green_pos,
                [0, 255, 0, 255],
                [150, 255, 150, 255],
                time + 0.5,
                scale_x,
                scale_y,
                x_offset,
                buffer_width,
                &draw_rays_fn,
                false,
            );
        }
    }
}

fn draw_ball_with_effects(
    frame: &mut [u8],
    width: u32,
    height: u32,
    pos: (f32, f32),
    ball_color: [u8; 4],
    ray_color: [u8; 4],
    time: f32,
    scale_x: f32,
    scale_y: f32,
    x_offset: usize,
    buffer_width: u32,
    draw_rays_fn: &impl Fn(&mut [u8], u32, u32, (f32, f32), [u8; 4], f32, usize, u32),
    is_yellow: bool,
) {
    draw_rays_fn(
        frame,
        width,
        height,
        pos,
        ray_color,
        time,
        x_offset,
        buffer_width,
    );

    // Get audio data for scaling - much more expressive scaling
    let mut audio_scale = 1.0;
    if let Some(spectrum) = get_audio_spectrum() {
        if let Ok(data) = spectrum.lock() {
            if !data.is_empty() {
                // Use different frequency ranges for each ball - swapped frequency ranges
                let audio_value = if is_yellow {
                    // Yellow ball responds to high frequencies (last quarter of spectrum)
                    let start = (data.len() * 3) / 4;
                    let end = data.len();
                    let mut high_avg = 0.0;
                    for i in start..end {
                        high_avg += data[i];
                    }
                    high_avg / (end - start) as f32
                } else {
                    // Green ball responds to bass frequencies (first quarter of spectrum)
                    let bass_range = data.len() / 4;
                    let mut bass_avg = 0.0;
                    for i in 0..bass_range {
                        bass_avg += data[i];
                    }
                    bass_avg / bass_range as f32
                };

                if is_yellow {
                    // Yellow ball: 10x more expressive scaling (normal level)
                    let enhanced_audio = audio_value.powf(0.5); // Square root for smoother scaling
                    audio_scale = 0.2 + enhanced_audio * 4.8; // Range: 0.2 to 5.0
                                                              // Add some dynamic pulsing based on audio peaks
                    let pulse_factor = (audio_value * 10.0).sin() * 0.3 + 1.0;
                    audio_scale *= pulse_factor;

                    // Remove size cap to allow unlimited ball growth
                    audio_scale = audio_scale.max(0.1);
                } else {
                    // Green ball: 100x more responsive but much smaller (extreme responsiveness, compact size)
                    let enhanced_audio = audio_value.powf(0.3); // Cube root for even more dramatic response
                    audio_scale = 0.3 + enhanced_audio * 2.7; // Range: 0.3 to 3.0 (much smaller range but same responsiveness)
                                                              // Add much more intense dynamic pulsing
                    let pulse_factor = (audio_value * 20.0).sin() * 0.8 + 1.0; // More intense pulsing
                    audio_scale *= pulse_factor;

                    // Remove size cap to allow unlimited ball growth
                    audio_scale = audio_scale.max(0.1);
                }
            }
        }
    }

    let base_ball_radius = 10.0 * scale_x.max(scale_y);
    let ball_radius = (base_ball_radius * audio_scale) as i32;
    draw_filled_circle(
        frame,
        width,
        height,
        pos.0 as i32,
        pos.1 as i32,
        ball_radius,
        &ball_color,
        x_offset,
        buffer_width,
    );

    // Remove glow effect completely - no more glow drawing
}

pub fn apply_force_yellow(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = BALL_STATE.as_mut().unwrap().yellow_vel {
            BALL_STATE.as_mut().unwrap().yellow_vel = Some((vel.0 + force_x, vel.1 + force_y));
        }
    }
}

pub fn apply_force_green(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = BALL_STATE.as_mut().unwrap().green_vel {
            BALL_STATE.as_mut().unwrap().green_vel = Some((vel.0 + force_x, vel.1 + force_y));
        }
    }
}

pub fn teleport_yellow(x: f32, y: f32) {
    unsafe {
        BALL_STATE.as_mut().unwrap().yellow_pos = Some((x, y));
    }
}

pub fn teleport_green(x: f32, y: f32) {
    unsafe {
        BALL_STATE.as_mut().unwrap().green_pos = Some((x, y));
    }
}
