use crate::{algorithms::sorter_manager, graphics::render, integration, physics};

pub fn draw_frame(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    let (scale_x, scale_y) = get_scale_factors(width, height);

    initialize_systems();
    physics::physics::update_physics(width, height, time, scale_x, scale_y);
    render::clear_frame(frame);
    draw_balls_and_rays(
        frame,
        width,
        height,
        time,
        scale_x,
        scale_y,
        x_offset,
        buffer_width,
    );
    sorter_manager::draw_sorter_visualizations(
        frame,
        width,
        height,
        time,
        scale_x,
        scale_y,
        x_offset,
        buffer_width,
    );
    sorter_manager::draw_algorithm_stats(frame, width, height, x_offset, buffer_width);
    integration::update_and_draw_audio(frame, width, height, time, x_offset, buffer_width);
    integration::update_and_draw_text(frame, width, height, time, x_offset, buffer_width);
}

fn get_scale_factors(_width: u32, _height: u32) -> (f32, f32) {
    let (monitor_width, monitor_height) = integration::get_monitor_dimensions();
    match (monitor_width, monitor_height) {
        (Some(m_width), Some(m_height)) => {
            let base_width = 1920.0;
            let base_height = 1080.0;
            (m_width as f32 / base_width, m_height as f32 / base_height)
        }
        _ => (1.0, 1.0),
    }
}

fn initialize_systems() {
    integration::initialize_audio_integration();
    integration::initialize_text_renderer();
    sorter_manager::initialize_sorters();
}

fn draw_balls_and_rays(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    scale_x: f32,
    scale_y: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    let (yellow_pos, green_pos) = physics::physics::get_ball_positions();

    if let (Some(yellow_pos), Some(green_pos)) = (yellow_pos, green_pos) {
        let draw_rays_closure = |frame: &mut [u8],
                                 width: u32,
                                 height: u32,
                                 pos: (f32, f32),
                                 ray_color: [u8; 4],
                                 time: f32,
                                 x_offset: usize,
                                 buffer_width: u32| {
            let other_pos = if pos == yellow_pos {
                green_pos
            } else {
                yellow_pos
            };
            render::draw_rays_from_ball(
                frame,
                width,
                height,
                pos,
                ray_color,
                time,
                x_offset,
                buffer_width,
                other_pos,
            );
        };

        physics::physics::draw_balls_with_effects(
            frame,
            width,
            height,
            time,
            scale_x,
            scale_y,
            x_offset,
            buffer_width,
            draw_rays_closure,
        );
    }
}
