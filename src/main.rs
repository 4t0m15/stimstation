use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod app;
mod audio_handler;
mod audio_integration;
mod audio_playback;
pub mod fibonacci_spiral;
mod menu;
mod mesmerise_circular;
mod particle_fountain;
mod pixel_utils;
pub mod pythagoras;
pub mod ray_pattern;
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

    let mut app = App::new(&window);
    
    // Initial draw to ensure the window shows something
    app.draw(pixels.frame_mut());
    if let Err(err) = pixels.render() {
        eprintln!("Initial render error: {err}");
        return Err(err);
    }
    window.request_redraw();

    event_loop
        .run(move |event, window_target| {
            // Set to Poll for continuous updates
            window_target.set_control_flow(ControlFlow::Poll);
            
            // Update input helper with the event
            if input.update(&event) {
                // Handle close requested
                if input.close_requested() || app.should_quit() {
                    window_target.exit();
                    return;
                }

                // Handle window resize
                if let Some(size) = input.window_resized() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Pixels resize error: {err}");
                        app.quit();
                        return;
                    }
                }

                // Update app logic
                app.handle_input(&mut input, &window);
                app.update();
                
                // Draw frame
                app.draw(pixels.frame_mut());

                // Render the frame
                if let Err(err) = pixels.render() {
                    eprintln!("Pixels render error: {err}");
                    app.quit();
                    return;
                }

                // Request redraw for continuous animation
                window.request_redraw();
            }
            
            // Handle redraw events
            match event {
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Update and render on every redraw
                    app.update();
                    app.draw(pixels.frame_mut());
                    
                    if let Err(err) = pixels.render() {
                        eprintln!("Pixels render error: {err}");
                        app.quit();
                        return;
                    }
                    
                    // Keep requesting redraws for animation
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();

    Ok(())
}
