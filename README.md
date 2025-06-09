# Mesmerise

A visual effects application built with Rust and Pixels that provides mesmerizing visualizations.

## Features

### Full-Screen Visualization
- Four different visual effects in a 2x2 grid layout:
  - Top-Left: Original line-based visualization
  - Top-Right: Circular visual patterns
  - Bottom-Left: Particle fountain with rainbow colors
  - Bottom-Right: Geometric patterns with animations

### Combined Visualization
- Both original and circular visualizations run side-by-side in a single window
- Toggle between visualization modes or view both at once
- Interactive controls for each visualization type

### Original Visualization
- Interactive line-based visuals that respond to mouse movements
- Multiple visual modes (Normal, Vortex, Waves, Rainbow)
- Particle explosion effects
- Adjustable line count

### Circular Visualization
- Three different circular visual effects in a split-screen layout:
  - Concentric rainbow circles
  - Orbiting green dots
  - Radial fan of white beams

### Ray Pattern Visualization
- Circular ray patterns with yellow and green light sources
- Physics-based movement with bouncing balls
- Realistic shadow effects when rays intersect with balls
- Sorting algorithm visualizations around the borders
- Random Wikipedia text fragments appearing throughout the screen

## Controls

### Main Controls
- **1**: Show only the original visualization
- **2**: Show only the circular visualization
- **3**: Show full-screen mode with all visualizations
- **4**: Show ray pattern visualization (default)
- **F11** or **Alt+Enter**: Toggle fullscreen mode
- **Escape**: Exit

### Original Visualization Controls (Left Side)
- **Space**: Toggle between visual modes (Normal, Vortex, Waves, Rainbow)
- **E**: Create explosion effect at center
- **F**: Create fireworks (multiple explosions)
- **+/-**: Increase/decrease line count
- **Mouse Move**: Influence line movement (left side only)
- **Left Mouse Button**: Create new lines (left side only)
- **Right Mouse Button**: Create explosion at cursor (left side only)

### Ray Pattern Controls
- **WASD**: Move yellow ball
- **Arrow Keys**: Move green ball
- **Left Mouse Button**: Teleport yellow ball to cursor position
- **Right Mouse Button**: Teleport green ball to cursor position

## Running the Application

```
# Run the main application in fullscreen mode with all visualizations
cargo run
```

The application starts in fullscreen mode showing the ray pattern visualization by default.

## Visual Proofs Examples

This project also includes several examples that demonstrate how to build visual proofs in Rust:

### 1. Static Pythagorean Theorem Diagram (Plotters)

Draws a diagram illustrating a² + b² = c² using the Plotters library.

```
# Run the Pythagorean theorem static diagram example
cargo run --example pythagoras_plot --features plotters
```

This generates a `pythagoras.png` file showing the relationship between the squares built on the sides of a right triangle.

### 2. Interactive Dissection Proof (Macroquad)

Animates the four congruent right triangles rotating to fill the big square.

```
# Run the interactive Pythagorean dissection proof
cargo run --example pythagoras_macroquad --features macroquad
```

This opens a window showing the dynamic demonstration of how the four triangles rotate within the square.

### 3. Induction Visualization (Egui + Plotters)

Uses a bar chart to show how the sum of the first n odd numbers builds up to n².

```
# Run the induction visualization
cargo run --example induction_egui --features "eframe plotters plotters-egui"
```

An interactive UI with a slider to adjust n and see how the sums of odd numbers equal perfect squares.

## Building

```
# Build the main application
cargo build

# Build with all visual proof examples
cargo build --features visual-proofs
```

## License

MIT 