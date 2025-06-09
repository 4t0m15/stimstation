// A simple visual proof of the formula 1 + 2 + 3 + ... + n = n(n+1)/2
// Demonstrates using basic ASCII output to show a triangular pattern

fn main() {
    let n = 10; // Size of triangle
    let sum = n * (n + 1) / 2;
    
    println!("Visual proof that 1 + 2 + 3 + ... + {} = {}*({} + 1)/2 = {}", n, n, n, sum);
    println!("=============================================");
    
    // Display the triangular number visually
    for i in 1..=n {
        // Print spaces for alignment
        for _ in 0..(n - i) {
            print!("  ");
        }
        
        // Print the dots representing numbers
        for _ in 0..i {
            print!("* ");
        }
        
        println!(" = {}", i);
    }
    
    println!("=============================================");
    println!("Total dots: {}", sum);
    
    // Show another proof by displaying the formula as a rectangle
    println!("\nAlternative proof: rearrange into a rectangle");
    println!("=============================================");
    
    // Create two triangles that form a rectangle
    for i in 1..=n {
        for j in 1..=i {
            print!("* ");
        }
        
        // Padding between triangles
        for _ in 0..(n - i) {
            print!("  ");
        }
        
        print!("| ");
        
        // Second triangle (in reverse)
        for j in 1..=n+1-i {
            print!("* ");
        }
        
        println!();
    }
    
    println!("=============================================");
    println!("Rectangle dimensions: {} × {}", n, n+1);
    println!("Rectangle area: {} × {} = {}", n, n+1, n*(n+1));
    println!("Triangle area (half of rectangle): {}/2 = {}", n*(n+1), sum);
} 