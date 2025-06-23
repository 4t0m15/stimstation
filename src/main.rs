use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod app;
mod audio_handler;
mod audio_playback;
mod fibonacci_spiral;
mod menu;
mod mesmerise_circular;
mod particle_fountain;
mod pixel_utils;
mod pythagoras;
mod ray_pattern;
mod rendering;
mod simple_proof;
mod text_rendering;
mod types;
mod visualizations;
mod world;

use crate::app::App;
use crate::types::{HEIGHT, WIDTH};

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("StimStation")
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
    };

    let mut app = App::new();

    event_loop
        .run(move |event, window_target| {
            if input.update(&event) {
                if input.close_requested() || app.should_quit() {
                    window_target.exit();
                    return;
                }

                if let Some(size) = input.window_resized() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Pixels resize error: {err}");
                        app.quit();
                        return;
                    }
                }

                app.handle_input(&mut input, &window);
                app.update();
                app.draw(&mut pixels);

                if let Err(err) = pixels.render() {
                    eprintln!("Pixels render error: {err}");
                    app.quit();
                    return;
                }

                window.request_redraw();
            }
        })
        .unwrap();

    Ok(())
}
