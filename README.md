# stimstation

A visual effects application built with Rust and Pixels that provides mesmerizing visualizations.

## Features

### Full-Screen Visualization
- Four different visual effects in a 2x2 grid layout:
  - Top-Left: Original line-based visualization
  - Top-Right: Circular visual patterns
  - Bottom-Left: Particle fountain with rainbow colors
  - Bottom-Right: Geometric patterns with animations

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
- **9**: Toggle white noise on/off
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

## License

MIT 