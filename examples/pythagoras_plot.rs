use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("pythagoras.png", (600, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Draw right triangle with sides a=3, b=4, c=5
    let triangle = vec![(50,50), (50,250), (290,50)];
    root.draw(&Polygon::new(triangle.clone(), BLUE.filled()))?;

    // Draw square on side a (vertical): side length 200
    let square_a = vec![(50,50), (50,250), (150,250), (150,50)];
    root.draw(&Polygon::new(square_a, RED.stroke_width(2)))?;

    // Draw square on side b (horizontal): side length 240
    let square_b = vec![(50,50), (290,50), (290,-190), (50,-190)];
    // shift up so it's visible
    let square_b: Vec<(i32,i32)> = square_b.into_iter().map(|(x,y)| (x, y + 300)).collect();
    root.draw(&Polygon::new(square_b, GREEN.stroke_width(2)))?;

    // Draw square on hypotenuse c (length ~360)
    let square_c = vec![(50,250),
                        (290,50),
                        (530,290),
                        (290,530)];
    root.draw(&Polygon::new(square_c, MAGENTA.stroke_width(2)))?;

    Ok(())
} 