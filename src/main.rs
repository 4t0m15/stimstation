use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    monitor::MonitorHandle,
};
use winit_input_helper::WinitInputHelper;
use stimstation::{
    types::{HEIGHT, WIDTH},
    orchestrator, integration,
};
use stimstation::app::App;
pub fn set_monitor_dimensions(monitor: &MonitorHandle) {
    integration::set_monitor_dimensions(monitor);
}

pub fn draw_frame(frame: &mut [u8], width: u32, height: u32, time: f32, x_offset: usize, buffer_width: u32) {
    orchestrator::draw_frame(frame, width, height, time, x_offset, buffer_width);
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("StimStation - Ray Pattern")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    });
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };    let mut app = App::new(&window);
    app.draw(pixels.frame_mut());
    if let Err(err) = pixels.render() {
        eprintln!("Initial render error: {err}");
        return Err(err);
    }
    window.request_redraw();    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);
            if input.update(&event) {
                if input.close_requested() || app.should_quit() {
                    window_target.exit();
                    return;                }
                if let Some(size) = input.window_resized() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Pixels resize error: {err}");
                        app.quit();
                        return;
                    }                }
                app.handle_input(&mut input, &window);
                app.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    eprintln!("Pixels render error: {err}");
                    app.quit();
                    return;                }
                window.request_redraw();
            }
            match event {
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..                } => {
                    app.draw(pixels.frame_mut());
                    if let Err(err) = pixels.render() {
                        eprintln!("Pixels render error: {err}");
                        app.quit();
                        return;                    }
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
    Ok(())
}
