            let px = x + (i as u32 % char_width);
            let py = y + (i as u32 / char_width);
            
            if px < frame_width && py < frame.len() as u32 / 4 / buffer_width {
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