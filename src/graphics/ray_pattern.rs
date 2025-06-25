use crate::{algorithms::sorter_manager, core::orchestrator, integration, physics};

pub fn set_monitor_dimensions(monitor: &winit::monitor::MonitorHandle) {
    integration::set_monitor_dimensions(monitor);
}

pub fn draw_frame(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    orchestrator::draw_frame(frame, width, height, time, x_offset, buffer_width);
}

pub fn apply_force_yellow(force_x: f32, force_y: f32) {
    physics::physics::apply_force_yellow(force_x, force_y);
}

pub fn apply_force_green(force_x: f32, force_y: f32) {
    physics::physics::apply_force_green(force_x, force_y);
}

pub fn teleport_yellow(x: f32, y: f32) {
    physics::physics::teleport_yellow(x, y);
}

pub fn teleport_green(x: f32, y: f32) {
    physics::physics::teleport_green(x, y);
}

pub fn restart_sorters() {
    sorter_manager::restart_sorters();
}
