// A visual demonstration of the Fibonacci sequence and the golden spiral
// Uses ASCII art to show how Fibonacci numbers form a spiral pattern

fn main() {
    // Calculate first few Fibonacci numbers
    let mut fibonacci = vec![1, 1];
    for i in 2..10 {
        fibonacci.push(fibonacci[i-1] + fibonacci[i-2]);
    }
    
    println!("Fibonacci Sequence Visual Proof");
    println!("===============================");
    println!("Fibonacci numbers: {:?}", fibonacci);
    println!("Each number is the sum of the two previous numbers.");
    println!();
    
    // Create a visual representation of the Fibonacci spiral using ASCII art
    let size = 50;
    let mut grid = vec![vec![' '; size]; size];
    
    // Draw the squares with size corresponding to Fibonacci numbers
    let mut x = size / 2;
    let mut y = size / 2;
    let mut direction = 0; // 0: right, 1: down, 2: left, 3: up
    
    for (i, &fib) in fibonacci.iter().enumerate() {
        let side = fib;
        
        // Draw the square borders
        match direction {
            0 => { // right
                for i in 0..side {
                    if x + i < size {
                        grid[y][x + i] = '+';
                        if y + side < size {
                            grid[y + side][x + i] = '+';
                        }
                    }
                }
                for i in 0..=side {
                    if y + i < size && x < size {
                        grid[y + i][x] = '+';
                        if x + side < size {
                            grid[y + i][x + side] = '+';
                        }
                    }
                }
                
                // Label with the Fibonacci number
                let label = fib.to_string();
                if x + side/2 < size && y + side/2 < size {
                    for (j, c) in label.chars().enumerate() {
                        if x + j + 1 < size {
                            grid[y + side/2][x + j + 1] = c;
                        }
                    }
                }
                
                // Move to next position
                if i > 0 {
                    x += side;
                }
            },
            1 => { // down
                for i in 0..side {
                    if y + i < size && x < size {
                        grid[y + i][x] = '+';
                        if x + side < size {
                            grid[y + i][x + side] = '+';
                        }
                    }
                }
                for i in 0..=side {
                    if x + i < size {
                        if y < size {
                            grid[y][x + i] = '+';
                        }
                        if y + side < size {
                            grid[y + side][x + i] = '+';
                        }
                    }
                }
                
                // Label with the Fibonacci number
                let label = fib.to_string();
                if x + side/2 < size && y + side/2 < size {
                    for (j, c) in label.chars().enumerate() {
                        if x + j + 1 < size {
                            grid[y + side/2][x + j + 1] = c;
                        }
                    }
                }
                
                // Move to next position
                y += side;
            },
            2 => { // left
                for i in 0..side {
                    if x >= i {
                        if y < size {
                            grid[y][x - i] = '+';
                        }
                        if y + side < size {
                            grid[y + side][x - i] = '+';
                        }
                    }
                }
                for i in 0..=side {
                    if y + i < size {
                        if x < size {
                            grid[y + i][x] = '+';
                        }
                        if x >= side {
                            grid[y + i][x - side] = '+';
                        }
                    }
                }
                
                // Label with the Fibonacci number
                let label = fib.to_string();
                if x >= side/2 + label.len() && y + side/2 < size {
                    for (j, c) in label.chars().enumerate() {
                        if x >= j + 1 {
                            grid[y + side/2][x - j - 1] = c;
                        }
                    }
                }
                
                // Move to next position
                x -= side;
            },
            3 => { // up
                for i in 0..side {
                    if y >= i {
                        if x < size {
                            grid[y - i][x] = '+';
                        }
                        if x + side < size {
                            grid[y - i][x + side] = '+';
                        }
                    }
                }
                for i in 0..=side {
                    if x + i < size {
                        if y < size {
                            grid[y][x + i] = '+';
                        }
                        if y >= side {
                            grid[y - side][x + i] = '+';
                        }
                    }
                }
                
                // Label with the Fibonacci number
                let label = fib.to_string();
                if x + side/2 < size && y >= side/2 {
                    for (j, c) in label.chars().enumerate() {
                        if x + j + 1 < size {
                            grid[y - side/2][x + j + 1] = c;
                        }
                    }
                }
                
                // Move to next position
                y -= side;
            },
            _ => {}
        }
        
        // Change direction for next square (clockwise)
        direction = (direction + 1) % 4;
    }
    
    // Draw the grid
    for row in grid {
        for cell in row {
            print!("{}", cell);
        }
        println!();
    }
    
    println!("\nThe spiral demonstrates how the Fibonacci sequence grows");
    println!("and approximates the golden ratio (φ ≈ 1.618...)");
    
    // Calculate and display the ratio of consecutive Fibonacci numbers
    println!("\nRatios of consecutive Fibonacci numbers:");
    for i in 1..fibonacci.len() {
        let ratio = fibonacci[i] as f64 / fibonacci[i-1] as f64;
        println!("{}/{} = {:.6}", fibonacci[i], fibonacci[i-1], ratio);
    }
    println!("\nAs the sequence progresses, the ratio approaches φ (1.618033988749895...)");
} 