use crate::algorithms::sorter::{SortVisualizer, SortAlgorithm, SortState, initialize_algorithm_stats};

static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;

pub fn initialize_sorters() {
    initialize_algorithm_stats();
    unsafe {
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new(SortAlgorithm::Merge));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new(SortAlgorithm::Insertion));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Selection));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Shell));
        }
    }
}

pub fn draw_sorter_visualizations(frame: &mut [u8], width: u32, height: u32, time: f32, 
                                 scale_x: f32, scale_y: f32, x_offset: usize, buffer_width: u32) {
    let scale_factor = (scale_x + scale_y) / 2.0;
    let border_thickness = (height as f32 * 0.05 * scale_factor) as usize;
    let side_width = (width as f32 * 0.05 * scale_factor) as usize;
    
    unsafe {
        update_and_draw_sorter(&mut TOP_SORTER, frame, 0, 0, width as usize, border_thickness, 
                              true, time, x_offset, buffer_width, false, true);  // flip_vertical = true for top
        update_and_draw_sorter(&mut BOTTOM_SORTER, frame, 0, height as usize - border_thickness, 
                              width as usize, border_thickness, true, time, x_offset, buffer_width, false, false);  // no flip for bottom
        update_and_draw_sorter(&mut LEFT_SORTER, frame, 0, border_thickness, side_width, 
                              height as usize - border_thickness * 2, false, time, x_offset, buffer_width, true, false);  // flip_horizontal = true for left
        update_and_draw_sorter(&mut RIGHT_SORTER, frame, width as usize - side_width, border_thickness, 
                              side_width, height as usize - border_thickness * 2, false, time, x_offset, buffer_width, false, false);  // no flip for right
    }
}

fn update_and_draw_sorter(sorter: &mut Option<SortVisualizer>, frame: &mut [u8], 
                         x: usize, y: usize, width: usize, height: usize, 
                         horizontal: bool, time: f32, x_offset: usize, buffer_width: u32,
                         flip_horizontal: bool, flip_vertical: bool) {
    if let Some(sorter) = sorter {
        sorter.update();
        if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
            sorter.restart();
        }
        sorter.draw_with_direction(frame, x, y, width, height, horizontal, x_offset, buffer_width as u32, flip_horizontal, flip_vertical);
    }
}

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

pub fn draw_algorithm_stats(frame: &mut [u8], width: u32, _height: u32, x_offset: usize, buffer_width: u32) {
    use crate::algorithms::sorter::get_algorithm_stats;
    
    if let Some(stats_arc) = get_algorithm_stats() {
        if let Ok(stats_map) = stats_arc.lock() {
            let mut stats_lines = Vec::new();
            
            // Collect all algorithm stats
            for (algorithm, count) in stats_map.iter() {
                stats_lines.push(format!("{}: {}", algorithm.name(), count));
            }
            
            // Sort by count (descending)
            stats_lines.sort_by(|a, b| {
                let a_count: u32 = a.split(": ").nth(1).unwrap_or("0").parse().unwrap_or(0);
                let b_count: u32 = b.split(": ").nth(1).unwrap_or("0").parse().unwrap_or(0);
                b_count.cmp(&a_count)
            });
            
            // Position the stats in the top-left, away from the top border sorter
            let stats_x = 10u32;
            let mut stats_y = 10u32;
            let char_width = 8u32;
            let char_height = 12u32;
            let line_spacing = 2u32;
            let padding = 4u32;
            
            // Calculate max text width
            let max_text_width = stats_lines.iter()
                .map(|line| line.len() as u32 * char_width)
                .max()
                .unwrap_or(0);
            
            let total_height = stats_lines.len() as u32 * (char_height + line_spacing) - line_spacing;
            
            // Draw semi-transparent dark background
            draw_background_rect(frame, stats_x - padding, stats_y - padding, 
                               max_text_width + padding * 2, total_height + padding * 2, 
                               [0, 0, 0, 180], width, x_offset, buffer_width);
            
            // Draw each line of text
            for line in stats_lines {
                draw_stats_text(frame, &line, stats_x, stats_y, [255, 255, 255, 255], width, x_offset, buffer_width);
                stats_y += char_height + line_spacing;
            }
        }
    }
}

fn draw_background_rect(frame: &mut [u8], x: u32, y: u32, width: u32, height: u32, 
                       color: [u8; 4], frame_width: u32, x_offset: usize, buffer_width: u32) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;
            
            if px < frame_width && py < frame.len() as u32 / 4 / buffer_width {
                let index = (((py * buffer_width + px + x_offset as u32) * 4) as usize).min(frame.len() - 4);
                if index + 3 < frame.len() {
                    // Alpha blend the background
                    let alpha = color[3] as f32 / 255.0;
                    let inv_alpha = 1.0 - alpha;
                    
                    frame[index] = (frame[index] as f32 * inv_alpha + color[0] as f32 * alpha) as u8;
                    frame[index + 1] = (frame[index + 1] as f32 * inv_alpha + color[1] as f32 * alpha) as u8;
                    frame[index + 2] = (frame[index + 2] as f32 * inv_alpha + color[2] as f32 * alpha) as u8;
                    frame[index + 3] = 255;
                }
            }
        }
    }
}

fn draw_stats_text(frame: &mut [u8], text: &str, x: u32, y: u32, color: [u8; 4], 
                   frame_width: u32, x_offset: usize, buffer_width: u32) {
    let char_width = 8;
    let char_height = 12;
    
    for (i, ch) in text.chars().enumerate() {
        let char_x = x + (i as u32 * char_width);
        draw_char(frame, ch, char_x, y, color, frame_width, char_width, char_height, x_offset, buffer_width);
    }
}

fn draw_char(frame: &mut [u8], ch: char, x: u32, y: u32, color: [u8; 4], 
             frame_width: u32, char_width: u32, char_height: u32, x_offset: usize, buffer_width: u32) {
    // Simple bitmap font for basic characters
    let pattern = get_char_pattern(ch);

    for (i, &pixel) in pattern.iter().enumerate() {
        if pixel > 0 {
            let px = x + (i as u32 % char_width);
            let py = y + (i as u32 / char_width);
            
            // Fixed bounds checking - calculate proper frame height
            let frame_height = frame.len() as u32 / 4 / buffer_width;
            if px < frame_width && py < frame_height {
                let index = (((py * buffer_width + px + x_offset as u32) * 4) as usize).min(frame.len() - 4);
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
    // 8x12 bitmap patterns for characters
    match ch {
        '0' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,1,
            1,1,0,0,1,0,1,1,
            1,1,0,1,0,0,1,1,
            1,1,1,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        '1' => vec![
            0,0,0,1,1,0,0,0,
            0,0,1,1,1,0,0,0,
            0,1,1,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            1,1,1,1,1,1,1,1,
        ],
        '2' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,1,1,0,
            0,0,0,0,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,
            0,1,1,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,1,
        ],
        '3' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,1,1,0,
            0,0,1,1,1,1,0,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        '4' => vec![
            0,0,0,0,0,1,1,0,
            0,0,0,0,1,1,1,0,
            0,0,0,1,1,1,1,0,
            0,0,1,1,0,1,1,0,
            0,1,1,0,0,1,1,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,1,1,0,
            1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
        ],
        '5' => vec![
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        '6' => vec![
            0,0,1,1,1,1,1,0,
            0,1,1,0,0,0,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,1,1,1,1,0,
            1,1,1,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,0,0,0,1,1,
            0,0,1,1,1,1,1,0,
        ],
        '7' => vec![
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,1,1,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,1,1,0,0,
            0,0,0,0,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,
            0,0,1,1,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,1,1,0,0,0,0,0,
        ],
        '8' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,0,0,1,1,0,
            0,0,1,1,1,1,0,0,
            0,1,1,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        '9' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,1,
            0,1,1,1,1,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            1,1,0,0,0,1,1,0,
            0,1,1,1,1,1,0,0,
        ],
        'A' => vec![
            0,0,1,1,1,1,0,0,
            0,1,1,0,0,1,1,0,
            0,1,1,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'B' => vec![
            1,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,0,
            1,1,1,1,1,1,0,0,
            1,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,0,
        ],
        'C' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        'D' => vec![
            1,1,1,1,1,1,0,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,0,
            1,1,1,1,1,1,0,0,
        ],
        'E' => vec![
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,1,1,1,1,0,0,
            1,1,1,1,1,1,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,1,1,1,1,1,1,
        ],
        'G' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,1,1,1,1,
            1,1,0,0,1,1,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        'I' => vec![
            1,1,1,1,1,1,1,1,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            1,1,1,1,1,1,1,1,
        ],
        'K' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,0,
            1,1,0,0,1,1,0,0,
            1,1,0,1,1,0,0,0,
            1,1,1,1,0,0,0,0,
            1,1,1,0,0,0,0,0,
            1,1,1,1,0,0,0,0,
            1,1,0,1,1,0,0,0,
            1,1,0,0,1,1,0,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
        ],
        'L' => vec![
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,1,
        ],
        'M' => vec![
            1,1,0,0,0,0,1,1,
            1,1,1,0,0,1,1,1,
            1,1,1,1,1,1,1,1,
            1,1,0,1,1,0,1,1,
            1,1,0,1,1,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'N' => vec![
            1,1,0,0,0,0,1,1,
            1,1,1,0,0,0,1,1,
            1,1,1,1,0,0,1,1,
            1,1,0,1,1,0,1,1,
            1,1,0,1,1,0,1,1,
            1,1,0,0,1,1,1,1,
            1,1,0,0,1,1,1,1,
            1,1,0,0,0,1,1,1,
            1,1,0,0,0,1,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'O' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        'Q' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,1,0,1,1,
            1,1,0,0,0,1,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,1,
        ],
        'R' => vec![
            1,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,1,1,0,
            1,1,1,1,1,1,0,0,
            1,1,1,1,1,1,0,0,
            1,1,0,0,1,1,0,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'S' => vec![
            0,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,0,1,1,1,1,0,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        'T' => vec![
            1,1,1,1,1,1,1,1,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
        ],
        'U' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        ':' => vec![
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
        ],
        'F' => vec![
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,1,1,1,1,0,0,
            1,1,1,1,1,1,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
        ],
        'H' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'J' => vec![
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,1,1,1,1,0,
        ],
        'P' => vec![
            1,1,1,1,1,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,1,1,1,1,1,0,
            1,1,1,1,1,1,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
        ],
        'V' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,0,0,1,1,0,
            0,1,1,0,0,1,1,0,
            0,0,1,1,1,1,0,0,
            0,0,1,1,1,1,0,0,
            0,0,0,1,1,0,0,0,
        ],
        'W' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,1,1,0,1,1,
            1,1,0,1,1,0,1,1,
            1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,
            1,1,1,0,0,1,1,1,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'X' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,0,0,1,1,0,
            0,1,1,0,0,1,1,0,
            0,0,1,1,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,1,1,0,0,
            0,1,1,0,0,1,1,0,
            0,1,1,0,0,1,1,0,
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
        ],
        'Y' => vec![
            1,1,0,0,0,0,1,1,
            1,1,0,0,0,0,1,1,
            0,1,1,0,0,1,1,0,
            0,1,1,0,0,1,1,0,
            0,0,1,1,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,1,1,0,0,0,
        ],
        'Z' => vec![
            1,1,1,1,1,1,1,1,
            0,0,0,0,0,0,1,1,
            0,0,0,0,0,1,1,0,
            0,0,0,0,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,
            0,1,1,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,0,0,0,0,0,0,
            1,1,1,1,1,1,1,1,
        ],
        ' ' => vec![0; 96], // Space
        '(' => vec![
            0,0,0,0,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,
            0,0,1,1,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,1,1,0,0,0,0,0,
            0,0,1,1,0,0,0,0,
            0,0,1,1,0,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,0,1,1,0,0,
        ],
        ')' => vec![
            0,0,1,1,0,0,0,0,
            0,0,0,1,1,0,0,0,
            0,0,0,0,1,1,0,0,
            0,0,0,0,1,1,0,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,0,1,1,0,
            0,0,0,0,1,1,0,0,
            0,0,0,0,1,1,0,0,
            0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,
        ],
        '-' => vec![
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            1,1,1,1,1,1,1,1,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
        ],
        _ => vec![1; 96], // Default block pattern for unknown characters
    }
}
