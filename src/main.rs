use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use stimstation::app::App;
use stimstation::types::{HEIGHT, WIDTH};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
    // Create the event loop and input helper
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    // Build the window
    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Welcome to StimStation!")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    });

    // Initialize the pixel buffer
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_size.width,
            window_size.height,
            Arc::clone(&window),
        );
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    // Create the app and perform initial draw
    let mut app = App::new(&window);
    app.draw(pixels.frame_mut());

    if let Err(err) = pixels.render() {
        eprintln!("Initial render error: {err}");
        return Err(err);
    }

    window.request_redraw();

    // Run the event loop
    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            // Handle input events
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
                app.draw(pixels.frame_mut());

                if let Err(err) = pixels.render() {
                    eprintln!("Pixels render error: {err}");
                    app.quit();
                    return;
                }

                window.request_redraw();
            }

            // Handle redraw requests
            match event {
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    app.draw(pixels.frame_mut());

                    if let Err(err) = pixels.render() {
                        eprintln!("Pixels render error: {err}");
                        app.quit();
                        return;
                    }

                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();

    Ok(())
}
