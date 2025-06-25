use pixels::{Pixels, SurfaceTexture};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub status: DownloadStatus,
    pub message: String,
}

#[derive(Clone, PartialEq)]
pub enum DownloadStatus {
    Starting,
    Downloading,
    Completed,
    Error,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            downloaded: 0,
            total: 0,
            status: DownloadStatus::Starting,
            message: "Initializing download...".to_string(),
        }
    }
}

pub async fn download_with_progress(
    url: &str,
    path: &PathBuf,
    progress: Arc<Mutex<DownloadProgress>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Update status to downloading
    {
        let mut p = progress.lock().unwrap();
        p.status = DownloadStatus::Downloading;
        p.message = "Connecting to server...".to_string();
    }

    let response = reqwest::get(url).await?;
    let total_size = response.content_length().unwrap_or(0);

    {
        let mut p = progress.lock().unwrap();
        p.total = total_size;
        p.message = "Downloading audio file...".to_string();
    }

    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = fs::File::create(path)?;
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    use futures::StreamExt;
    use std::io::Write;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        // Update progress
        {
            let mut p = progress.lock().unwrap();
            p.downloaded = downloaded;
        }
    }

    // Mark as completed
    {
        let mut p = progress.lock().unwrap();
        p.status = DownloadStatus::Completed;
        p.message = "Download completed successfully!".to_string();
    }

    Ok(())
}

fn draw_progress_window(pixels: &mut Pixels, progress: &Arc<Mutex<DownloadProgress>>) {
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    let frame = pixels.frame_mut();

    // Clear background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 20; // R
        pixel[1] = 20; // G
        pixel[2] = 30; // B
        pixel[3] = 255; // A
    }

    if let Ok(progress) = progress.lock() {
        draw_progress_bar(frame, width, height, &progress);
        draw_text(frame, width, height, &progress);
    }
}

fn draw_progress_bar(frame: &mut [u8], width: u32, height: u32, progress: &DownloadProgress) {
    let bar_x = 50;
    let bar_y = height / 2 - 10;
    let bar_width = width - 100;
    let bar_height = 20;

    // Draw background bar
    draw_rectangle(
        frame,
        bar_x,
        bar_y,
        bar_width,
        bar_height,
        [60, 60, 70, 255],
        width,
    );

    // Draw progress
    if progress.total > 0 {
        let progress_ratio = progress.downloaded as f32 / progress.total as f32;
        let progress_width = (bar_width as f32 * progress_ratio) as u32;

        let color = match progress.status {
            DownloadStatus::Starting => [100, 100, 200, 255],
            DownloadStatus::Downloading => [100, 200, 100, 255],
            DownloadStatus::Completed => [100, 255, 100, 255],
            DownloadStatus::Error => [255, 100, 100, 255],
        };

        if progress_width > 0 {
            draw_rectangle(
                frame,
                bar_x,
                bar_y,
                progress_width,
                bar_height,
                color,
                width,
            );
        }
    }

    // Draw border
    draw_rectangle_outline(
        frame,
        bar_x,
        bar_y,
        bar_width,
        bar_height,
        [150, 150, 150, 255],
        width,
    );
}

fn draw_text(frame: &mut [u8], width: u32, height: u32, progress: &DownloadProgress) {
    // Draw status message
    let message_y = height / 2 - 40;
    draw_simple_text(
        frame,
        &progress.message,
        50,
        message_y,
        [200, 200, 200, 255],
        width,
    );

    // Draw progress percentage
    if progress.total > 0 {
        let percentage = (progress.downloaded as f64 / progress.total as f64 * 100.0) as u32;
        let progress_text = format!(
            "{}% ({:.1} MB / {:.1} MB)",
            percentage,
            progress.downloaded as f64 / 1024.0 / 1024.0,
            progress.total as f64 / 1024.0 / 1024.0
        );
        let progress_y = height / 2 + 35;
        draw_simple_text(
            frame,
            &progress_text,
            50,
            progress_y,
            [180, 180, 180, 255],
            width,
        );
    }
}

fn draw_rectangle(
    frame: &mut [u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
    frame_width: u32,
) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;
            if px < frame_width && py < frame.len() as u32 / 4 / frame_width {
                let index = ((py * frame_width + px) * 4) as usize;
                if index + 3 < frame.len() {
                    frame[index] = color[0];
                    frame[index + 1] = color[1];
                    frame[index + 2] = color[2];
                    frame[index + 3] = color[3];
                }
            }
        }
    }
}

fn draw_rectangle_outline(
    frame: &mut [u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
    frame_width: u32,
) {
    // Top and bottom edges
    for dx in 0..width {
        let px = x + dx;
        if px < frame_width {
            // Top edge
            let index_top = ((y * frame_width + px) * 4) as usize;
            if index_top + 3 < frame.len() {
                frame[index_top] = color[0];
                frame[index_top + 1] = color[1];
                frame[index_top + 2] = color[2];
                frame[index_top + 3] = color[3];
            }

            // Bottom edge
            let py_bottom = y + height - 1;
            let index_bottom = ((py_bottom * frame_width + px) * 4) as usize;
            if index_bottom + 3 < frame.len() {
                frame[index_bottom] = color[0];
                frame[index_bottom + 1] = color[1];
                frame[index_bottom + 2] = color[2];
                frame[index_bottom + 3] = color[3];
            }
        }
    }

    // Left and right edges
    for dy in 0..height {
        let py = y + dy;
        if py < frame.len() as u32 / 4 / frame_width {
            // Left edge
            let index_left = ((py * frame_width + x) * 4) as usize;
            if index_left + 3 < frame.len() {
                frame[index_left] = color[0];
                frame[index_left + 1] = color[1];
                frame[index_left + 2] = color[2];
                frame[index_left + 3] = color[3];
            }

            // Right edge
            let px_right = x + width - 1;
            let index_right = ((py * frame_width + px_right) * 4) as usize;
            if index_right + 3 < frame.len() {
                frame[index_right] = color[0];
                frame[index_right + 1] = color[1];
                frame[index_right + 2] = color[2];
                frame[index_right + 3] = color[3];
            }
        }
    }
}

fn draw_simple_text(
    frame: &mut [u8],
    text: &str,
    x: u32,
    y: u32,
    color: [u8; 4],
    frame_width: u32,
) {
    let char_width = 8;
    let char_height = 12;

    for (i, ch) in text.chars().enumerate() {
        let char_x = x + (i as u32 * char_width);
        draw_char(
            frame,
            ch,
            char_x,
            y,
            color,
            frame_width,
            char_width,
            char_height,
        );
    }
}

fn draw_char(
    frame: &mut [u8],
    ch: char,
    x: u32,
    y: u32,
    color: [u8; 4],
    frame_width: u32,
    char_width: u32,
    _char_height: u32,
) {
    // Simple bitmap font for basic characters
    let pattern = get_char_pattern(ch);

    for (i, &pixel) in pattern.iter().enumerate() {
        if pixel > 0 {
            let px = x + (i as u32 % char_width);
            let py = y + (i as u32 / char_width);

            if px < frame_width && py < frame.len() as u32 / 4 / frame_width {
                let index = ((py * frame_width + px) * 4) as usize;
                if index + 3 < frame.len() {
                    frame[index] = color[0];
                    frame[index + 1] = color[1];
                    frame[index + 2] = color[2];
                    frame[index + 3] = color[3];
                }
            }
        }
    }
}

fn get_char_pattern(ch: char) -> Vec<u8> {
    // Simple bitmap patterns for common characters
    match ch {
        'A'..='Z' | 'a'..='z' => vec![1; 96], // Simple block for letters
        '0'..='9' => vec![1; 96],             // Simple block for numbers
        ' ' => vec![0; 96],                   // Space
        '.' | '%' | '(' | ')' | '/' | '-' | ':' => vec![1; 96], // Simple block for symbols
        _ => vec![1; 96],                     // Default block
    }
}

// Global flag to track if we're already showing a download window
static DOWNLOAD_WINDOW_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static ERROR_WINDOW_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn show_download_progress(
    url: &str,
    path: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Force reset the flag at the start to handle any stale state
    DOWNLOAD_WINDOW_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);

    // Check if we're already showing a download window to prevent multiple EventLoops
    if DOWNLOAD_WINDOW_ACTIVE
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        println!("Download window already active, retrying...");
        // Wait a moment and try again
        thread::sleep(Duration::from_millis(100));
        DOWNLOAD_WINDOW_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
        if DOWNLOAD_WINDOW_ACTIVE
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return Err("Download window still active after retry".into());
        }
    }

    // Ensure we reset the flag when this function exits
    struct FlagGuard;
    impl Drop for FlagGuard {
        fn drop(&mut self) {
            println!("Resetting download window flag");
            DOWNLOAD_WINDOW_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let _guard = FlagGuard;

    println!("Starting download progress window for: {}", url);

    use std::sync::mpsc;

    // Create a channel to communicate between threads
    let (tx, rx) = mpsc::channel();

    // Spawn the download in a separate thread with proper Tokio runtime
    let download_url = url.to_string();
    let download_path = path.clone();
    let progress_handle = Arc::new(Mutex::new(DownloadProgress::default()));
    let download_progress = Arc::clone(&progress_handle);
    thread::spawn(move || {
        // Create a new Tokio runtime for this thread
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let mut p = download_progress.lock().unwrap();
                p.status = DownloadStatus::Error;
                p.message = format!("Failed to create async runtime: {}", e);
                let _ = tx.send(());
                return;
            }
        };

        // Run the download within the Tokio runtime
        rt.block_on(async {
            if let Err(e) =
                download_with_progress(&download_url, &download_path, download_progress.clone())
                    .await
            {
                let mut p = download_progress.lock().unwrap();
                p.status = DownloadStatus::Error;
                p.message = format!("Download failed: {}", e);
            }
        });
        // Signal that download thread is done
        let _ = tx.send(());
    });
    // Create and run the progress window in the main thread
    println!("Creating event loop for progress window...");
    let event_loop = EventLoop::new()?;

    println!("Event loop created successfully");

    // Get monitor dimensions for 50% sizing
    let (window_width, window_height) = if let Some(monitor) = event_loop.primary_monitor() {
        let size = monitor.size();
        println!("Monitor size: {}x{}", size.width, size.height);
        println!("Monitor size: {}x{}", size.width, size.height);
        (size.width / 2, size.height / 2)
    } else {
        println!("No primary monitor found, using fallback size");
        (800, 600) // Fallback size
    };

    println!(
        "Creating window with size: {}x{}",
        window_width, window_height
    );

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("StimStation - Downloading Audio")
            .with_inner_size(LogicalSize::new(window_width as f64, window_height as f64))
            .with_resizable(false)
            .with_decorations(false) // Remove window borders and title bar
            .build(&event_loop)?,
    );

    println!("Window created successfully");

    // Create pixels renderer
    let window_size = window.inner_size();
    println!(
        "Setting up pixels renderer with size: {}x{}",
        window_size.width, window_size.height
    );
    println!(
        "Setting up pixels renderer with size: {}x{}",
        window_size.width, window_size.height
    );
    let surface_texture =
        SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
    let mut pixels = Pixels::new(window_size.width, window_size.height, surface_texture)?;

    println!("Pixels renderer created, starting event loop...");

    let mut last_check = std::time::Instant::now();
    let mut completion_start: Option<std::time::Instant> = None;
    let error_to_show = Arc::new(Mutex::new(None::<String>));
    let error_to_show_clone = Arc::clone(&error_to_show);

    // Run the event loop
    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                draw_progress_window(&mut pixels, &progress_handle);
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {err}");
                    window_target.exit();
                }
            }
            _ => {}
        }

        // Check if download is complete (throttled to avoid excessive checking)
        if last_check.elapsed() > Duration::from_millis(100) {
            if let Ok(progress) = progress_handle.lock() {
                match progress.status {
                    DownloadStatus::Completed => {
                        if completion_start.is_none() {
                            completion_start = Some(std::time::Instant::now());
                        } else if completion_start.unwrap().elapsed() > Duration::from_millis(1500)
                        {
                            window_target.exit();
                        }
                    }
                    DownloadStatus::Error => {
                        if completion_start.is_none() {
                            completion_start = Some(std::time::Instant::now());
                            if let Ok(mut error_msg) = error_to_show_clone.lock() {
                                *error_msg = Some(progress.message.clone());
                            }
                        } else if completion_start.unwrap().elapsed() > Duration::from_millis(2000)
                        {
                            window_target.exit();
                        }
                    }
                    _ => {
                        completion_start = None; // Reset if status changes back
                    }
                }
            }

            // Check if download thread finished
            if rx.try_recv().is_ok() {
                // Download thread finished, continue showing progress
            }

            last_check = std::time::Instant::now();
        }

        window.request_redraw();

        // Use a consistent control flow
        window_target.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(16),
        ));
    })?; // Check if there was an error and show error window
    if let Ok(error_opt) = error_to_show.lock() {
        if let Some(error_msg) = error_opt.clone() {
            eprintln!("Download failed: {}", error_msg);
            if let Err(e) = show_error_window(error_msg) {
                eprintln!("Failed to show error window: {}", e);
            }
            return Err("Download failed - see error window for details".into());
        }
    }

    // Verify download completed successfully
    if path.exists() {
        Ok(path.clone())
    } else {
        let error_msg = "Download failed - file not found after download".to_string();
        if let Err(e) = show_error_window(error_msg.clone()) {
            eprintln!("Failed to show error window: {}", e);
        }
        Err(error_msg.into())
    }
}

pub fn show_error_window(error_message: String) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we're already showing an error window to prevent multiple EventLoops
    if ERROR_WINDOW_ACTIVE
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        eprintln!(
            "Error window already active, printing error to console: {}",
            error_message
        );
        return Ok(());
    }

    // Ensure we reset the flag when this function exits
    struct ErrorFlagGuard;
    impl Drop for ErrorFlagGuard {
        fn drop(&mut self) {
            ERROR_WINDOW_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let _guard = ErrorFlagGuard;
    // Create and run the error window
    let event_loop = EventLoop::new()?;

    // Get monitor dimensions for 50% sizing
    let (window_width, window_height) = if let Some(monitor) = event_loop.primary_monitor() {
        let size = monitor.size();
        (size.width / 2, size.height / 2)
    } else {
        (800, 600) // Fallback size
    };

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("StimStation - Download Error")
            .with_inner_size(LogicalSize::new(window_width as f64, window_height as f64))
            .with_resizable(false)
            .with_decorations(false) // Remove window borders and title bar
            .build(&event_loop)?,
    );

    // Create pixels renderer
    let window_size = window.inner_size();
    let surface_texture =
        SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
    let mut pixels = Pixels::new(window_size.width, window_size.height, surface_texture)?;

    let start_time = std::time::Instant::now();

    // Run the event loop
    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                draw_error_window(&mut pixels, &error_message);
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {err}");
                    window_target.exit();
                }
            }
            _ => {}
        }

        // Auto-close after 5 seconds, but allow user to close manually
        if start_time.elapsed() > Duration::from_millis(5000) {
            window_target.exit();
        }

        window.request_redraw();

        window_target.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(16),
        ));
    })?;

    Ok(())
}

fn draw_error_window(pixels: &mut Pixels, error_message: &str) {
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    let frame = pixels.frame_mut();

    // Clear background with dark red tint
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 40; // R
        pixel[1] = 20; // G
        pixel[2] = 20; // B
        pixel[3] = 255; // A
    }

    // Draw error border
    draw_rectangle_outline(
        frame,
        10,
        10,
        width - 20,
        height - 20,
        [200, 100, 100, 255],
        width,
    );
    draw_rectangle_outline(
        frame,
        12,
        12,
        width - 24,
        height - 24,
        [200, 100, 100, 255],
        width,
    );

    // Draw error title
    draw_simple_text(frame, "DOWNLOAD ERROR", 50, 30, [255, 150, 150, 255], width);

    // Draw error message (split into lines if too long)
    let max_chars_per_line = 50;
    let mut y_offset = 70;
    let words: Vec<&str> = error_message.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > max_chars_per_line {
            if !current_line.is_empty() {
                draw_simple_text(
                    frame,
                    &current_line,
                    30,
                    y_offset,
                    [200, 200, 200, 255],
                    width,
                );
                y_offset += 20;
                current_line.clear();
            }
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    // Draw remaining line
    if !current_line.is_empty() {
        draw_simple_text(
            frame,
            &current_line,
            30,
            y_offset,
            [200, 200, 200, 255],
            width,
        );
        y_offset += 20;
    }

    // Draw instructions
    draw_simple_text(
        frame,
        "This window will close automatically in 5 seconds",
        30,
        y_offset + 20,
        [180, 180, 180, 255],
        width,
    );
    draw_simple_text(
        frame,
        "or click the X to close manually",
        30,
        y_offset + 40,
        [180, 180, 180, 255],
        width,
    );
}
