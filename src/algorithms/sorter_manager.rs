use crate::algorithms::sorter::{
    get_algorithm_stats, initialize_algorithm_stats, SortAlgorithm, SortState, SortVisualizer,
};
use crate::physics::detect_corner;

// Global static sorters - each positioned in different areas of the screen
static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;

pub fn initialize_sorters() {
    initialize_algorithm_stats();
    // Use a fixed size for fair comparison - all algorithms sort the same number of elements
    // This ensures the leaderboard is based on algorithm speed, not array size differences
    const FIXED_ARRAY_SIZE: usize = 100;
    unsafe {
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new_with_size(SortAlgorithm::Shell, FIXED_ARRAY_SIZE));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new_with_size(SortAlgorithm::Quick, FIXED_ARRAY_SIZE));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new_with_size(SortAlgorithm::Insertion, FIXED_ARRAY_SIZE));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new_with_size(SortAlgorithm::Selection, FIXED_ARRAY_SIZE));
        }
    }
}

pub fn draw_sorter_visualizations(
    frame: &mut [u8],
    width: u32,
    height: u32,
    time: f32,
    scale_x: f32,
    scale_y: f32,
    x_offset: usize,
    buffer_width: u32,
) {
    let scale_factor = (scale_x + scale_y) / 2.0;
    let border_thickness = (height as f32 * 0.05 * scale_factor) as usize;
    let side_width = (width as f32 * 0.15 * scale_factor) as usize;

    unsafe {
        update_and_draw_sorter(
            &mut TOP_SORTER,
            frame,
            0,
            0,
            width as usize,
            border_thickness,
            true,
            time,
            x_offset,
            buffer_width,
            false,
            true,
        ); // flip_vertical = true for top
        update_and_draw_sorter(
            &mut BOTTOM_SORTER,
            frame,
            0,
            height as usize - border_thickness,
            width as usize,
            border_thickness,
            true,
            time,
            x_offset,
            buffer_width,
            false,
            false,
        ); // no flip for bottom
        update_and_draw_sorter(
            &mut LEFT_SORTER,
            frame,
            0,
            border_thickness,
            side_width,
            height as usize - border_thickness * 2,
            false,
            time,
            x_offset,
            buffer_width,
            true,
            false,
        ); // flip_horizontal = true for left
        update_and_draw_sorter(
            &mut RIGHT_SORTER,
            frame,
            width as usize - side_width,
            border_thickness,
            side_width,
            height as usize - border_thickness * 2,
            false,
            time,
            x_offset,
            buffer_width,
            false,
            false,
        ); // no flip for right
    }
}

fn update_and_draw_sorter(
    sorter: &mut Option<SortVisualizer>,
    frame: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    horizontal: bool,
    time: f32,
    x_offset: usize,
    buffer_width: u32,
    flip_horizontal: bool,
    flip_vertical: bool,
) {
    if let Some(sorter) = sorter {
        sorter.update();
        if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
            sorter.restart();
        }
        sorter.draw_with_direction(
            frame,
            x,
            y,
            width,
            height,
            horizontal,
            x_offset,
            buffer_width as u32,
            flip_horizontal,
            flip_vertical,
        );
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

pub fn draw_algorithm_stats(
    frame: &mut [u8],
    width: u32,
    _height: u32,
    x_offset: usize,
    buffer_width: u32,
) {
    if let Some(stats_arc) = get_algorithm_stats() {
        if let Ok(stats_map) = stats_arc.lock() {
            // Collect and sort algorithms by completion count
            let mut stats_vec: Vec<(SortAlgorithm, u32)> = stats_map
                .iter()
                .map(|(alg, &cnt)| (alg.clone(), cnt))
                .collect();
            stats_vec.sort_by(|a, b| b.1.cmp(&a.1));
            // Only keep the top 4 algorithms for display
            stats_vec.truncate(4);

            let char_width = 8;
            let char_height = 12;
            let _padding = 4;
            let stats_x = _padding;
            let stats_y = 10u32;

            // Calculate background dimensions based on longest text
            let max_len = stats_vec
                .iter()
                .map(|(alg, count)| format!("{}: {}", alg.name(), count).len())
                .max()
                .unwrap_or(0) as u32;
            let bg_width = max_len * char_width + _padding * 2;
            let bg_height = (char_height + 2) * stats_vec.len() as u32 + _padding * 2;

            // Draw background for leaderboard
            draw_background_rect(
                frame,
                stats_x - _padding,
                stats_y - _padding,
                bg_width,
                bg_height,
                [0, 0, 0, 180],
                width,
                x_offset,
                buffer_width,
            );

            // Draw each algorithm entry
            for (i, (alg, count)) in stats_vec.iter().enumerate() {
                let entry_text = format!("{}: {}", alg.name(), count);
                let text_y = stats_y + i as u32 * (char_height + 2);
                draw_stats_text(
                    frame,
                    &entry_text,
                    stats_x,
                    text_y,
                    [255, 255, 255, 255],
                    width,
                    x_offset,
                    buffer_width,
                );
            }

            // Draw corner hits below leaderboard
            let corner_hits = detect_corner::get_corner_hits();
            let corner_text = format!("{} corner hits", corner_hits);
            let corner_y = stats_y + (stats_vec.len() as u32 * (char_height + 2)) + _padding;
            let ct_height = char_height;
            draw_background_rect(
                frame,
                stats_x - _padding,
                corner_y - _padding,
                bg_width,
                ct_height + _padding * 2,
                [0, 0, 0, 180],
                width,
                x_offset,
                buffer_width,
            );
            draw_stats_text(
                frame,
                &corner_text,
                stats_x,
                corner_y,
                [255, 255, 255, 255],
                width,
                x_offset,
                buffer_width,
            );
        }
    }
}

fn draw_background_rect(
    frame: &mut [u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
    frame_width: u32,
    x_offset: usize,
    buffer_width: u32,
) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;

            if px < frame_width && py < frame.len() as u32 / 4 / buffer_width {
                let index = (((py * buffer_width + px + x_offset as u32) * 4) as usize)
                    .min(frame.len() - 4);
                if index + 3 < frame.len() {
                    // Alpha blend the background
                    let alpha = color[3] as f32 / 255.0;
                    let inv_alpha = 1.0 - alpha;

                    frame[index] =
                        (frame[index] as f32 * inv_alpha + color[0] as f32 * alpha) as u8;
                    frame[index + 1] =
                        (frame[index + 1] as f32 * inv_alpha + color[1] as f32 * alpha) as u8;
                    frame[index + 2] =
                        (frame[index + 2] as f32 * inv_alpha + color[2] as f32 * alpha) as u8;
                    frame[index + 3] = 255;
                }
            }
        }
    }
}

fn draw_stats_text(
    frame: &mut [u8],
    text: &str,
    x: u32,
    y: u32,
    color: [u8; 4],
    frame_width: u32,
    x_offset: usize,
    buffer_width: u32,
) {
    let char_width = 8;
    let char_height = 12;
    let _padding = 4;

    // Draw each character in the text
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
            x_offset,
            buffer_width,
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
    x_offset: usize,
    buffer_width: u32,
) {
    // Simple bitmap font for basic characters
    let pattern = get_char_pattern(ch);

    for (i, &pixel) in pattern.iter().enumerate() {
        if pixel > 0 {
            let px = x + (i as u32 % char_width);
            let py = y + (i as u32 / char_width);

            // Fixed bounds checking - calculate proper frame height
            let frame_height = frame.len() as u32 / 4 / buffer_width;
            if px < frame_width && py < frame_height {
                let index = (((py * buffer_width + px + x_offset as u32) * 4) as usize)
                    .min(frame.len() - 4);
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
    // Convert lowercase letters to uppercase for pattern matching
    let ch = ch.to_ascii_uppercase();
    // 8x12 bitmap patterns for characters
    match ch {
        ' ' => vec![0; 96], // Empty space
        ':' => vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        '0' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
            1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1,
            0, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        '1' => vec![
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        '2' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        '3' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        '4' => vec![
            0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0,
            1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0,
        ],
        '5' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        '6' => vec![
            0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1,
            1, 0, 0, 1, 1, 1, 1, 1, 0,
        ],
        '7' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 0, 0, 0,
        ],
        '8' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        '9' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1,
            1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1,
            0, 0, 1, 1, 1, 1, 1, 0, 0,
        ],
        'A' => vec![
            0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'B' => vec![
            1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 0,
        ],
        'C' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        'D' => vec![
            1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1,
            0, 1, 1, 1, 1, 1, 1, 0, 0,
        ],
        'E' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        'F' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 0,
        ],
        'G' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        'H' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'I' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        'J' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1,
            0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        'K' => vec![
            1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0,
            0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1,
            0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'L' => vec![
            1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        'M' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'N' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'O' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        'P' => vec![
            1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 0,
        ],
        'Q' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 1, 1,
        ],
        'R' => vec![
            1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1,
            0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'S' => vec![
            0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        'T' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 1, 1, 0, 0, 0,
        ],
        'U' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 0, 1, 1, 1, 1, 1, 1, 0,
        ],
        'V' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
            0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
            1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0,
            0, 0, 0, 1, 1, 1, 1, 0, 0,
        ],
        'W' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1,
            0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1,
            0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'X' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1,
            1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        'Y' => vec![
            1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1,
            1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0,
            1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0,
            0, 0, 0, 1, 1, 1, 1, 0, 0,
        ],
        'Z' => vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        '(' => vec![
            0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0,
            0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 1, 1, 0, 0,
        ],
        ')' => vec![
            0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0, 0,
        ],
        '-' => vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        _ => vec![1; 96], // Default to a block for undefined characters
    }
}
