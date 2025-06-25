use crate::audio::audio_integration::AudioIntegration;
use crate::text::text_processor::TextProcessor;
use winit::monitor::MonitorHandle;

static mut AUDIO_INTEGRATION: Option<AudioIntegration> = None;
static mut TEXT_RENDERER: Option<TextProcessor> = None;
static mut MONITOR_WIDTH: Option<u32> = None;
static mut MONITOR_HEIGHT: Option<u32> = None;

pub fn set_monitor_dimensions(monitor: &MonitorHandle) {
    let size = monitor.size();
    unsafe {
        MONITOR_WIDTH = Some(size.width);
        MONITOR_HEIGHT = Some(size.height);
        println!("Monitor dimensions set: {}x{}", size.width, size.height);
    }
}

pub fn get_monitor_dimensions() -> (Option<u32>, Option<u32>) {
    unsafe { (MONITOR_WIDTH, MONITOR_HEIGHT) }
}

pub fn initialize_audio_integration() {
    unsafe {
        if AUDIO_INTEGRATION.is_none() {
            AUDIO_INTEGRATION = Some(AudioIntegration::new());
        }
        if let Some(audio_integration) = AUDIO_INTEGRATION.as_mut() {
            audio_integration.initialize();
        }
    }
}

pub fn update_and_draw_audio(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    unsafe {
        if let Some(audio_integration) = AUDIO_INTEGRATION.as_mut() {
            let monitor_height = MONITOR_HEIGHT;
            audio_integration.update(time, monitor_height);
            audio_integration.draw(frame, width, height, x_offset, buffer_width);
        }
    }
}

pub fn initialize_text_renderer() {}

pub fn update_and_draw_text(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    unsafe {
        if let Some(text_renderer) = TEXT_RENDERER.as_mut() {
            text_renderer.update(time, width, height);
            text_renderer.draw(frame, width, height, x_offset, buffer_width);
        }
    }
}
