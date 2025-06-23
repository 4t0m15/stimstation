// Ray pattern visualization inspired by the yellow and green rays image
use ab_glyph::{Font as _, FontArc, Glyph, PxScale};
use font_kit::{source::SystemSource, properties::Properties};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::sync::Once;
use winit::monitor::MonitorHandle;
use crate::audio_handler::AudioVisualizer;
use crate::audio_playback::{start_audio_thread, is_audio_thread_started};

// Initialize once for thread safety
static INIT: Once = Once::new();

const RAY_COUNT: usize = 60;  // Number of rays for each source

// Sorting visualization data
const SORT_ARRAY_SIZE: usize = 200; // A more reasonable size that won't cause overflow
struct SortVisualizer {
    array: Vec<u8>,
    steps: usize,
    algorithm: SortAlgorithm,
    state: SortState,
    i: usize,
    j: usize,
    pivot: usize,
    stack: Vec<(usize, usize)>, // For quicksort
    comparisons: usize,
    accesses: usize,
}

#[derive(Debug, PartialEq)]
enum SortAlgorithm {
    Bogo,
    Bubble,
    Quick,
}

#[derive(Debug, PartialEq)]
enum SortState {
    Running,
    Completed,
    Restarting,
}

impl SortVisualizer {
    fn new(algorithm: SortAlgorithm) -> Self {
        let mut array = Vec::with_capacity(SORT_ARRAY_SIZE);
        for i in 1..=SORT_ARRAY_SIZE {
            array.push((i % 255) as u8);
        }
        
        let mut rng = thread_rng();
        array.shuffle(&mut rng);
        
        Self {
            array,
            steps: 0,
            algorithm,
            state: SortState::Running,
            i: 0,
            j: 0,
            pivot: 0,
            stack: Vec::new(),
            comparisons: 0,
            accesses: 0,
        }
    }
    
    fn update(&mut self) {
        if self.state == SortState::Completed {
            return;
        }
        
        if self.state == SortState::Restarting {
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            
            self.state = SortState::Running;
            self.steps = 0;
            self.i = 0;
            self.j = 0;
            self.comparisons = 0;
            self.accesses = 0;
            self.stack.clear();
            
            if self.algorithm == SortAlgorithm::Quick {
                self.stack.push((0, self.array.len() - 1));
            }
            
            return;
        }
        
        match self.algorithm {
            SortAlgorithm::Bogo => self.update_bogo(),
            SortAlgorithm::Bubble => self.update_bubble(),
            SortAlgorithm::Quick => self.update_quick(),
        }
        
        self.steps += 1;
    }
    
    fn update_bogo(&mut self) {
        let mut is_sorted = true;
        for i in 1..self.array.len() {
            self.accesses += 2;
            if self.array[i-1] > self.array[i] {
                is_sorted = false;
                break;
            }
        }
        
        if is_sorted {
            self.state = SortState::Completed;
        } else {
            let mut rng = thread_rng();
            self.array.shuffle(&mut rng);
            self.accesses += self.array.len() * 2;
        }
    }
    
    fn update_bubble(&mut self) {
        // Bubble sort implementation
        // One step of bubble sort
        let n = self.array.len();
        
        // Check if we've completed a full pass
        if self.i >= n {
            // Check if any swaps were made in this pass
            if self.j == 0 {
                // No swaps were made, array is sorted
                self.state = SortState::Completed;
            } else {
                // Start a new pass
                self.i = 0;
                self.j = 0;
            }
            return;
        }
        
        // Compare adjacent elements
        if self.i < n - 1 {
            self.comparisons += 1;
            self.accesses += 2; // Reading two elements
            
            if self.array[self.i] > self.array[self.i + 1] {
                // Swap elements
                self.array.swap(self.i, self.i + 1);
                self.accesses += 2; // Writing two elements
                self.j += 1; // Count the swap
            }
        }
        
        // Move to next comparison
        self.i += 1;
    }
    
    fn update_quick(&mut self) {
        // Quick sort implementation (iterative version using a stack)
        if self.stack.is_empty() {
            self.state = SortState::Completed;
            return;
        }
        
        // Pop a subarray from stack
        let (low, high) = self.stack.pop().unwrap();
        
        // If there's only one element in this subarray, continue
        if low >= high {
            return;
        }
        
        // Partition the array
        let pivot = self.partition(low, high);
        
        // Push subarrays to stack
        if pivot > 0 && pivot - 1 > low {
            self.stack.push((low, pivot - 1));
        }
        if pivot + 1 < high {
            self.stack.push((pivot + 1, high));
        }
    }
    
    fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.array[high];
        self.accesses += 1;
        
        let mut i = low;
        
        for j in low..high {
            self.comparisons += 1;
            self.accesses += 1;
            
            if self.array[j] <= pivot {
                self.array.swap(i, j);
                self.accesses += 4;
                i += 1;
            }
        }
        
        self.array.swap(i, high);
        self.accesses += 4;
        
        i
    }
    
    fn draw(&self, frame: &mut [u8], x: usize, y: usize, width: usize, height: usize, 
           horizontal: bool, x_offset: usize, buffer_width: u32) {
        let n = self.array.len();
        
        let sorted_percent = self.get_sorted_percent();
        let r = ((1.0 - sorted_percent) * 255.0) as u8;
        let g = (sorted_percent * 255.0) as u8;
        let b = 50;
        
        if horizontal {
            let bar_width = width as f32 / n as f32;
            
            for i in 0..n {
                let bar_height = (self.array[i] as f32 / 255.0 * height as f32) as usize;
                
                let color = if self.state == SortState::Running &&
                               ((self.algorithm == SortAlgorithm::Bubble && (i == self.i || i == self.i + 1)) ||
                                (self.algorithm == SortAlgorithm::Quick && i == self.pivot)) {
                    [255, 255, 0]
                } else {
                    [r, g, b]
                };
                
                for j in 0..bar_height {
                    for k in 0..(bar_width as usize) {
                        let mut current_height = height as u32;
                        if let Some(mon_h) = unsafe { MONITOR_HEIGHT } {
                            current_height = mon_h;
                        }

                        put_pixel_safe(frame, (x + (i as f32 * bar_width) as usize + k) as i32, (y + height - j - 1) as i32, buffer_width, current_height, [color[0], color[1], color[2], 255], x_offset);
                    }
                }
            }
        } else {
            let bar_height = height as f32 / n as f32;
            
            for i in 0..n {
                let bar_width = (self.array[i] as f32 / 255.0 * width as f32) as usize;
                
                let color = if self.state == SortState::Running &&
                               ((self.algorithm == SortAlgorithm::Bubble && (i == self.i || i == self.i + 1)) ||
                                (self.algorithm == SortAlgorithm::Quick && i == self.pivot)) {
                    [255, 255, 0]
                } else {
                    [r, g, b]
                };
                
                for j in 0..bar_width {
                    for k in 0..(bar_height as usize) {
                         let mut current_height = height as u32;
                        if let Some(mon_h) = unsafe { MONITOR_HEIGHT } {
                            current_height = mon_h;
                        }
                        put_pixel_safe(frame, (x + j) as i32, (y + (i as f32 * bar_height) as usize + k) as i32, buffer_width, current_height, [color[0], color[1], color[2], 255], x_offset);
                    }
                }
            }
        }
    }
    
    fn get_sorted_percent(&self) -> f32 {
        let mut sorted_count = 0;
        
        for i in 1..self.array.len() {
            if self.array[i-1] <= self.array[i] {
                sorted_count += 1;
            }
        }
        
        sorted_count as f32 / (self.array.len() - 1) as f32
    }
    
    fn restart(&mut self) {
        self.state = SortState::Restarting;
    }
}

// Static instance of audio visualizer (using external module)
static mut AUDIO_VISUALIZER: Option<crate::audio_handler::AudioVisualizer> = None;

// Static instances for sorting visualizers
static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;

// HSV to RGB conversion helper for colorful visualization
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8
    ]
}

struct TextFragment {
    text: String,
    x: i32,
    y: i32,
    color: [u8; 3],
    life: f32,
    max_life: f32,
    scale: f32,
}

static mut TEXT_FRAGMENTS: Option<Vec<TextFragment>> = None;
static mut NEXT_FRAGMENT_TIME: Option<f32> = None;
static mut MONITOR_WIDTH: Option<u32> = None;
static mut MONITOR_HEIGHT: Option<u32> = None;

static FONT: Lazy<FontArc> = Lazy::new(|| {
    let source = SystemSource::new();
    let handle = source
        .select_best_match(&[font_kit::family_name::FamilyName::Monospace], &Properties::new())
        .expect("Failed to find a monospace font");

    match handle.load() {
        Ok(font) => {
            let font_data = font.copy_font_data().expect("Failed to get font data");
            FontArc::try_from_vec(font_data.to_vec()).expect("Failed to convert font to FontArc")
        },
        Err(_) => panic!("Failed to load font"),
    }
});

pub fn set_monitor_dimensions(monitor: &MonitorHandle) {
    let size = monitor.size();
    unsafe {
        MONITOR_WIDTH = Some(size.width);
        MONITOR_HEIGHT = Some(size.height);
        println!("Monitor dimensions set: {}x{}", size.width, size.height);
    }
}

pub fn draw_frame(frame: &mut [u8], width: u32, height: u32, time: f32, x_offset: usize, buffer_width: u32) {
    // Get scale factors based on monitor dimensions
    let (scale_x, scale_y) = unsafe {
        match (MONITOR_WIDTH, MONITOR_HEIGHT) {
            (Some(m_width), Some(m_height)) => {
                // Scale based on monitor dimensions
                // Use a reasonable base size for reference
                let base_width = 1920.0;
                let base_height = 1080.0;
                (m_width as f32 / base_width, m_height as f32 / base_height)
            },
            _ => (1.0, 1.0) // Default scale if monitor dimensions aren't available
        }
    };
    // Initialize text fragments if needed
    unsafe {
        if TEXT_FRAGMENTS.is_none() {
            TEXT_FRAGMENTS = Some(Vec::with_capacity(20));
            NEXT_FRAGMENT_TIME = Some(time + 0.5); // Start adding fragments sooner
        }
        
        // Initialize audio visualizer if needed
        if AUDIO_VISUALIZER.is_none() {
            AUDIO_VISUALIZER = Some(AudioVisualizer::new());
        }
        
        // Start audio thread if it hasn't been started yet
        if !is_audio_thread_started() {
            // Start the audio thread for noise generation and analysis
            if let Some(_handle) = start_audio_thread() {
                println!("Audio thread started successfully");
            }
        }
        
        // Initialize sorting visualizers if needed
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bubble));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bogo));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        
        // Add new text fragments occasionally
        if let Some(next_time) = NEXT_FRAGMENT_TIME {
            if time >= next_time {
                // Add a new text fragment
                add_random_text_fragment(width, height, time);
                
                // Schedule next fragment - more frequent appearance (0.2-1.0 seconds)
                let mut rng = rand::thread_rng();
                let delay = rng.gen_range(0.2..1.0);
                NEXT_FRAGMENT_TIME = Some(time + delay);
            }
        }
        
        // Initialize ball positions if they haven't been yet
        if YELLOW_POS.is_none() {
            let quarter_width = width as f32 / 4.0;
            let quarter_height = height as f32 / 4.0;
            
            // Scale ball velocities based on screen size
            let vel_scale = (scale_x + scale_y) / 2.0;
            let base_vel_x = 1.0 * vel_scale;
            let base_vel_y = 0.5 * vel_scale;
            
            // Position yellow ball in the upper-left quadrant
            YELLOW_POS = Some((quarter_width * 1.5, quarter_height * 1.5));
            YELLOW_VEL = Some((base_vel_x, base_vel_y));
            
            // Position green ball in the lower-right quadrant
            GREEN_POS = Some((width as f32 - quarter_width * 1.5, height as f32 - quarter_height * 1.5));
            GREEN_VEL = Some((-base_vel_x, -base_vel_y));
        }
        
        // Update ball positions
        let dt = if let Some(last) = LAST_TIME {
            let delta = time - last;
            // Limit delta time to avoid huge jumps if the app freezes
            if delta > 0.1 { 0.1 } else { delta }
        } else {
            0.016 // ~60 FPS if no previous time
        };
        
        LAST_TIME = Some(time);
        
        // Update yellow ball position
        if let (Some(pos), Some(vel)) = (YELLOW_POS.as_mut(), YELLOW_VEL.as_mut()) {
            // Scale movement speed based on screen size
            let speed_scale = (scale_x + scale_y) / 2.0;
            let base_speed = 50.0 * speed_scale;
            
            // Update position
            pos.0 += vel.0 * base_speed * dt;
            pos.1 += vel.1 * base_speed * dt;
            
            // Bounce off walls
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
        
        // Update green ball position
        if let (Some(pos), Some(vel)) = (GREEN_POS.as_mut(), GREEN_VEL.as_mut()) {
            // Scale movement speed based on screen size
            let speed_scale = (scale_x + scale_y) / 2.0;
            let base_speed = 50.0 * speed_scale;
            
            // Update position
            pos.0 += vel.0 * base_speed * dt;
            pos.1 += vel.1 * base_speed * dt;
            
            // Bounce off walls
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
            
            // Ball collision detection with yellow ball
            if let Some(yellow_pos) = YELLOW_POS {
                let dx = pos.0 - yellow_pos.0;
                let dy = pos.1 - yellow_pos.1;
                let dist_sq = dx * dx + dy * dy;
                
                // Scale collision distance based on screen size
                let min_dist = 30.0 * ((scale_x + scale_y) / 2.0);
                
                if dist_sq < min_dist * min_dist {
                    // They're colliding, calculate new velocities
                    let dist = dist_sq.sqrt();
                    let nx = dx / dist;
                    let ny = dy / dist;
                    
                    // Move them apart to prevent sticking
                    pos.0 = yellow_pos.0 + nx * min_dist;
                    pos.1 = yellow_pos.1 + ny * min_dist;
                    
                    if let Some(yellow_vel) = YELLOW_VEL {
                        // Elastic collision
                        let dot_prod = vel.0 * nx + vel.1 * ny - yellow_vel.0 * nx - yellow_vel.1 * ny;
                        
                        // Update velocities
                        vel.0 -= dot_prod * nx;
                        vel.1 -= dot_prod * ny;
                        
                        if let Some(yellow_vel_mut) = YELLOW_VEL.as_mut() {
                            yellow_vel_mut.0 += dot_prod * nx;
                            yellow_vel_mut.1 += dot_prod * ny;
                        }
                    }
                }
            }
        }
    }
    
    // Clear frame with a dark background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 5; // R
        pixel[1] = 5; // G
        pixel[2] = 10; // B
        pixel[3] = 255; // A
    }
    
    // Draw rays for yellow and green light sources
    unsafe {
        // Draw rays from both light sources
        if let Some(yellow_pos) = YELLOW_POS {
            let rays_yellow = [255, 255, 150, 255]; // Yellow color
            draw_rays(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                width as i32 / 2, height as i32 / 2,
                width as i32 / 2 - 20, &rays_yellow,
                RAY_COUNT, time, x_offset, buffer_width
            );
            
            // Draw the yellow light source (scaled to screen size)
            let yellow_color = [255, 255, 0, 255];
            let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
            draw_filled_circle(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                ball_radius, &yellow_color,
                x_offset, buffer_width
            );
            
            // Add glow around the yellow source (scaled to screen size)
            let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
            draw_shadow_glow(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                glow_radius, &[255, 255, 100, 100],
                x_offset, buffer_width
            );
        }
        
        if let Some(green_pos) = GREEN_POS {
            let rays_green = [150, 255, 150, 255]; // Green color
            draw_rays(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                width as i32 / 2, height as i32 / 2,
                width as i32 / 2 - 20, &rays_green,
                RAY_COUNT, time + 0.5, x_offset, buffer_width
            );
            
            // Draw the green light source (scaled to screen size)
            let green_color = [0, 255, 0, 255];
            let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
            draw_filled_circle(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                ball_radius, &green_color,
                x_offset, buffer_width
            );
            
            // Add glow around the green source (scaled to screen size)
            let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
            draw_shadow_glow(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                glow_radius, &[100, 255, 100, 100],
                x_offset, buffer_width
            );
        }
    }
    
    unsafe {
        // Update and draw the sorting visualizers along the edges of the screen
        // Scale the border thickness based on screen dimensions
        let scale_factor = (scale_x + scale_y) / 2.0;
        let border_thickness = (height as f32 * 0.05 * scale_factor) as usize; // 5% of height for top/bottom
        let side_width = (width as f32 * 0.05 * scale_factor) as usize; // 5% of width for sides
        
        // Update the sorters
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw top sorter
            sorter.draw(
                frame, 
                0, 0, // x, y position 
                width as usize, border_thickness, // width, height
                true, // horizontal
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw bottom sorter
            sorter.draw(
                frame,
                0, height as usize - border_thickness, // x, y position
                width as usize, border_thickness, // width, height
                true, // horizontal
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw left sorter
            sorter.draw(
                frame,
                0, border_thickness, // x, y position
                side_width, height as usize - border_thickness * 2, // width, height
                false, // vertical
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw right sorter
            sorter.draw(
                frame,
                width as usize - side_width, border_thickness, // x, y position
                side_width, height as usize - border_thickness * 2, // width, height
                false, // vertical
                x_offset, buffer_width
            );
        }
        
        // Update and draw the audio visualizer
        if let Some(audio_viz) = AUDIO_VISUALIZER.as_mut() {
            let monitor_height = MONITOR_HEIGHT;
            audio_viz.update(time, monitor_height);
            audio_viz.draw(frame, width, height, x_offset, buffer_width);
        }
        
        // Update and draw text fragments
        if let Some(ref mut fragments) = TEXT_FRAGMENTS {
            // Update fragment lifetimes
            fragments.retain_mut(|f| {
                f.life -= 1.0 / 60.0; // Assuming 60 FPS
                f.life > 0.0
            });

            // Add new fragments
            add_random_text_fragment(width, height, time);

            // Draw fragments
            for fragment in fragments.iter() {
                draw_text_fragment(frame, width, height, fragment, x_offset, buffer_width);
            }
        }
    }
}

// Draw rays from a source point around a circle
fn draw_rays(frame: &mut [u8], width: u32, height: u32, 
            source_x: i32, source_y: i32, 
            center_x: i32, center_y: i32,
            radius: i32, color: &[u8; 4], 
            count: usize, time: f32, 
            x_offset: usize, buffer_width: u32) {
    
    // Get the current position of the other ball to check for shadows
    let (other_x, other_y, other_radius) = unsafe {
        if source_x == YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32 && 
           source_y == YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32 {
            // This is the yellow ball, so get green ball position
            (GREEN_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             GREEN_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10) // Ball radius
        } else {
            // This is the green ball, so get yellow ball position
            (YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10) // Ball radius
        }
    };

    // Store shadow rays for drawing later
    let mut shadow_rays: Vec<((i32, i32), (i32, i32))> = Vec::new();
    
    for i in 0..count {
        // Calculate angle with small animation
        let base_angle = (i as f32 / count as f32) * 2.0 * std::f32::consts::PI;
        let angle = base_angle + (time * 0.2).sin() * 0.05; // Small oscillation
        
        // Calculate endpoint on circle
        let end_x = center_x as f32 + angle.cos() * radius as f32;
        let end_y = center_y as f32 + angle.sin() * radius as f32;
        
        // Check if ray intersects with the other ball
        // Using ray-circle intersection test
        let ray_dir_x = end_x as f32 - source_x as f32;
        let ray_dir_y = end_y as f32 - source_y as f32;
        let ray_length = (ray_dir_x * ray_dir_x + ray_dir_y * ray_dir_y).sqrt();
        
        // Normalize ray direction
        let ray_dir_x = ray_dir_x / ray_length;
        let ray_dir_y = ray_dir_y / ray_length;
        
        // Vector from ray origin to circle center
        let oc_x = source_x as f32 - other_x as f32;
        let oc_y = source_y as f32 - other_y as f32;
        
        // Quadratic formula components
        let a = 1.0; // Since ray direction is normalized
        let b = 2.0 * (ray_dir_x * oc_x + ray_dir_y * oc_y);
        let c = (oc_x * oc_x + oc_y * oc_y) - (other_radius * other_radius) as f32;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant >= 0.0 {
            // Ray intersects the sphere
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
            
            // Check if intersection is along the ray (not behind it)
            if (t1 > 0.0 && t1 < ray_length) || (t2 > 0.0 && t2 < ray_length) {
                // Calculate intersection point (using the first intersection)
                let t = t1.max(0.0);
                let intersect_x = (source_x as f32 + ray_dir_x * t) as i32;
                let intersect_y = (source_y as f32 + ray_dir_y * t) as i32;
                
                // Draw ray only up to the intersection point
                draw_line(frame, width, height, source_x, source_y, intersect_x, intersect_y, 
                         color, x_offset, buffer_width);
                
                // Calculate shadow ray - extends from the intersection point to beyond the circle
                // Make the shadow ray extend to the edge of the main circle
                let shadow_length = radius as f32 * 1.2; // Longer than the circle radius
                let shadow_end_x = (intersect_x as f32 + ray_dir_x * shadow_length) as i32;
                let shadow_end_y = (intersect_y as f32 + ray_dir_y * shadow_length) as i32;
                
                // Store shadow ray for later drawing
                shadow_rays.push(((intersect_x, intersect_y), (shadow_end_x, shadow_end_y)));
            } else {
                // No intersection, draw full ray
                draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                         color, x_offset, buffer_width);
            }
        } else {
            // No intersection, draw full ray
            draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                     color, x_offset, buffer_width);
        }
    }
    
    // Draw shadow rays with faded color
    let shadow_color = [
        (color[0] as f32 * 0.2) as u8,
        (color[1] as f32 * 0.2) as u8,
        (color[2] as f32 * 0.2) as u8,
        128, // Semi-transparent
    ];
    
    for shadow in shadow_rays {
        draw_line(frame, width, height, 
                 shadow.0.0, shadow.0.1, 
                 shadow.1.0, shadow.1.1, 
                 &shadow_color, x_offset, buffer_width);
    }
}

// Draw a filled circle
fn draw_filled_circle(frame: &mut [u8], width: u32, height: u32, 
                     center_x: i32, center_y: i32, 
                     radius: i32, color: &[u8; 4],
                     x_offset: usize, buffer_width: u32) {
    
    for y in -radius..=radius {
        let h = (radius.pow(2) - y.pow(2)) as f32;
        if h < 0.0 { continue; }
        
        let w = h.sqrt() as i32;
        for x in -w..=w {
            put_pixel(frame, width, height, center_x + x, center_y + y, color, x_offset, buffer_width);
        }
    }
}

// Draw a line using Bresenham's algorithm
fn draw_line(frame: &mut [u8], width: u32, height: u32, 
            x0: i32, y0: i32, 
            x1: i32, y1: i32, 
            color: &[u8; 4],
            x_offset: usize, buffer_width: u32) {
    
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        put_pixel(frame, width, height, x0, y0, color, x_offset, buffer_width);
        
        if x0 == x1 && y0 == y1 { break; }
        
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 { break; }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 { break; }
            err += dx;
            y0 += sy;
        }
    }
}

// Helper to put a pixel in the frame buffer
fn put_pixel(frame: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: &[u8; 4], 
            x_offset: usize, buffer_width: u32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    
    // Calculate the index in the buffer with the x_offset
    let x_pos = x as usize + x_offset;
    let idx = ((y as u32 * buffer_width + x_pos as u32) * 4) as usize;
    
    if idx + 3 < frame.len() {
        // Apply alpha blending
        frame[idx] = (frame[idx] as u16 * (255 - color[3]) as u16 / 255 + color[0] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 1] = (frame[idx + 1] as u16 * (255 - color[3]) as u16 / 255 + color[1] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 2] = (frame[idx + 2] as u16 * (255 - color[3]) as u16 / 255 + color[2] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 3] = 255;
    }
}

// Public functions to apply forces to the balls
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

pub fn teleport_yellow(x: f32, y: f32) { unsafe { YELLOW_POS = Some((x, y)); } }
pub fn teleport_green(x: f32, y: f32) { unsafe { GREEN_POS = Some((x, y)); } }

// Draw a shadow glow effect
fn draw_shadow_glow(frame: &mut [u8], width: u32, height: u32, 
                   center_x: i32, center_y: i32, 
                   radius: i32, color: &[u8; 4],
                   x_offset: usize, buffer_width: u32) {
    let radius_sq = radius * radius;
    let inner_radius_sq = (radius - 5).max(0) * (radius - 5).max(0); // Soft inner edge
    
    // Define the bounding box of the glow
    let x_min = (center_x - radius).max(0);
    let x_max = (center_x + radius).min(width as i32 - 1);
    let y_min = (center_y - radius).max(0);
    let y_max = (center_y + radius).min(height as i32 - 1);
    
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let dx = x - center_x;
            let dy = y - center_y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < radius_sq {
                // Calculate intensity based on distance from center
                let intensity = if dist_sq > inner_radius_sq {
                    // Fade out from inner to outer radius
                    1.0 - (dist_sq as f32 - inner_radius_sq as f32) / (radius_sq as f32 - inner_radius_sq as f32)
                } else {
                    1.0
                };
                
                // Mix with a darker version for shadow effect
                let shadow_color = [
                    (color[0] as f32 * 0.5) as u8,
                    (color[1] as f32 * 0.5) as u8,
                    (color[2] as f32 * 0.5) as u8,
                    (color[3] as f32 * intensity) as u8
                ];
                
                // Blend with existing pixels for a softer glow
                let idx = 4 * (y as usize * buffer_width as usize + (x as usize + x_offset));
                if idx + 3 < frame.len() {
                    let existing_r = frame[idx];
                    let existing_g = frame[idx + 1];
                    let existing_b = frame[idx + 2];
                    
                    frame[idx] = existing_r.saturating_add(shadow_color[0]);
                    frame[idx + 1] = existing_g.saturating_add(shadow_color[1]);
                    frame[idx + 2] = existing_b.saturating_add(shadow_color[2]);
                    frame[idx + 3] = 255;
                }
            }
        }
    }
}

// Add a new random text fragment
fn add_random_text_fragment(width: u32, height: u32, _time: f32) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < 0.1 {
        unsafe {
            if let Some(fragments) = TEXT_FRAGMENTS.as_mut() {
                if fragments.len() > 50 {
                    return;
                }

                let _segment_length = rng.gen_range(20..80);
                let text = format!("Text fragment {}", rng.gen_range(1..1000));

                let x = rng.gen_range(0..width as i32);
                let y = rng.gen_range(0..height as i32);

                let scale = rng.gen_range(20.0..40.0);
                let color = hsv_to_rgb(rng.gen_range(0.0..1.0), 0.8, 1.0);
                let lifetime = rng.gen_range(5.0..15.0);

                fragments.push(TextFragment {
                    text,
                    x,
                    y,
                    color,
                    life: lifetime,
                    max_life: lifetime,
                    scale,
                });
            }
        }
    }
}

// Draw a text fragment with fading effect
fn draw_text_fragment(
    frame: &mut [u8],
    _width: u32,
    height: u32,
    fragment: &TextFragment,
    x_offset: usize,
    buffer_width: u32,
) {
    let alpha = (fragment.life / fragment.max_life * 255.0) as u8;
    let color = [fragment.color[0], fragment.color[1], fragment.color[2], alpha];

    // Use ab_glyph to draw the text
    let scale = PxScale::from(fragment.scale);
    let glyphs = layout_paragraph(&FONT, scale, fragment.text.as_str());

    for glyph in glyphs {
        if let Some(outlined) = FONT.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|px, py, v| {
                let gx = px as i32 + bounds.min.x as i32;
                let gy = py as i32 + bounds.min.y as i32;

                let pixel_x = fragment.x + gx;
                let pixel_y = fragment.y + gy;

                let final_alpha = (v * (alpha as f32 / 255.0) * 255.0) as u8;

                put_pixel_safe(
                    frame,
                    pixel_x,
                    pixel_y,
                    buffer_width,
                    height,
                    [color[0], color[1], color[2], final_alpha],
                    x_offset,
                );
            });
        }
    }
}

fn layout_paragraph(font: &FontArc, scale: PxScale, text: &str) -> Vec<Glyph> {
    let mut glyphs = Vec::new();
    let mut x = 0.0;
    let mut y = 0.0;
    
    for c in text.chars() {
        if c == '\n' {
            x = 0.0;
            y += scale.y;
            continue;
        }
        
        let glyph_id = font.glyph_id(c);
        let mut positioned = glyph_id.with_scale(scale);
        positioned.position = ab_glyph::point(x, y);
        glyphs.push(positioned);
        x += font.h_advance_unscaled(glyph_id) * scale.x;
    }
    
    glyphs
}

// In the put_pixel_safe function, add blending for smoother text
fn put_pixel_safe(
    frame: &mut [u8],
    x: i32,
    y: i32,
    buffer_width: u32,
    buffer_height: u32,
    color: [u8; 4],
    x_offset: usize,
) {
    if x >= 0 && x < buffer_width as i32 && y >= 0 && y < buffer_height as i32 {
        let x_global = x as usize + x_offset;
        let y_global = y as usize;

        let idx = 4 * (y_global * buffer_width as usize + x_global);
        if idx + 3 < frame.len() {
            // Simple alpha blending
            let bg_r = frame[idx] as f32;
            let bg_g = frame[idx + 1] as f32;
            let bg_b = frame[idx + 2] as f32;

            let fg_r = color[0] as f32;
            let fg_g = color[1] as f32;
            let fg_b = color[2] as f32;
            let fg_a = color[3] as f32 / 255.0;

            frame[idx] = ((fg_r * fg_a) + (bg_r * (1.0 - fg_a))) as u8;
            frame[idx + 1] = ((fg_g * fg_a) + (bg_g * (1.0 - fg_a))) as u8;
            frame[idx + 2] = ((fg_b * fg_a) + (bg_b * (1.0 - fg_a))) as u8;
            frame[idx + 3] = 255;
        }
    }
}

// Public function to restart all sorting visualizers
pub fn restart_sorters() {
    unsafe {
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.restart();
        }
    }
}

static mut YELLOW_POS: Option<(f32, f32)> = None;
static mut YELLOW_VEL: Option<(f32, f32)> = None;
static mut GREEN_POS: Option<(f32, f32)> = None;
static mut GREEN_VEL: Option<(f32, f32)> = None;
static mut LAST_TIME: Option<f32> = None; 