/// Represents a simplified SVG path command.
/// For Phase 1, we define a basic set.
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo { x: f64, y: f64 },
    LineTo { x: f64, y: f64 },
    ClosePath,
    // Add CurveTo, etc. as needed later
}

/// Parses an SVG path string into structured commands.
pub fn parse_svg_path(path_str: &str) -> Result<Vec<PathCommand>, String> {
    let mut commands = Vec::new();
    let tokens: Vec<&str> = path_str.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        match token {
            "M" => {
                if i + 2 >= tokens.len() { return Err("M command missing coordinates".to_string()); }
                let x = tokens[i+1].parse::<f64>().map_err(|_| "Invalid x coord")?;
                let y = tokens[i+2].parse::<f64>().map_err(|_| "Invalid y coord")?;
                commands.push(PathCommand::MoveTo { x, y });
                i += 3;
            },
            "L" => {
                if i + 2 >= tokens.len() { return Err("L command missing coordinates".to_string()); }
                let x = tokens[i+1].parse::<f64>().map_err(|_| "Invalid x coord")?;
                let y = tokens[i+2].parse::<f64>().map_err(|_| "Invalid y coord")?;
                commands.push(PathCommand::LineTo { x, y });
                i += 3;
            },
            "Z" => {
                commands.push(PathCommand::ClosePath);
                i += 1;
            },
            _ => return Err(format!("Unknown command or unexpected token: {}", token)),
        }
    }
    Ok(commands)
}

/// Rasterizes a list of SVG paths into a grid mask.
pub fn rasterize_obstacles(width: usize, height: usize, svg_paths: &[String]) -> Vec<f64> {
    let mut grid = vec![0.0; width * height];
    for path in svg_paths {
        rasterize_path(width, height, path, &mut grid);
    }
    grid
}

/// Rasterizes a single SVG path string onto the material grid.
pub fn rasterize_path(width: usize, height: usize, path: &str, grid: &mut [f64]) {
    if let Ok(commands) = parse_svg_path(path) {
        fill_path_on_grid(width, height, &commands, grid);
    } else {
        // In a real app we might log error, here we ignore invalid paths as per simplicity
    }
}

/// Fills the shape defined by the commands onto the grid.
pub fn fill_path_on_grid(width: usize, height: usize, commands: &[PathCommand], grid: &mut [f64]) {
    // Simple Ray-Casting algorithm for point-in-polygon
    // 1. Convert commands to vertices
    let mut vertices = Vec::new();
    for cmd in commands {
        match cmd {
            PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => vertices.push((*x, *y)),
            _ => {}
        }
    }
    if vertices.is_empty() { return; }

    // Optimization: Bounding Box
    let min_x = vertices.iter().fold(f64::INFINITY, |a, v| a.min(v.0)).floor() as isize;
    let max_x = vertices.iter().fold(f64::NEG_INFINITY, |a, v| a.max(v.0)).ceil() as isize;
    let min_y = vertices.iter().fold(f64::INFINITY, |a, v| a.min(v.1)).floor() as isize;
    let max_y = vertices.iter().fold(f64::NEG_INFINITY, |a, v| a.max(v.1)).ceil() as isize;

    // Clamp bounding box to grid
    let start_x = min_x.max(0) as usize;
    let end_x = max_x.min(width as isize) as usize;
    let start_y = min_y.max(0) as usize;
    let end_y = max_y.min(height as isize) as usize;

    for y in start_y..end_y {
        for x in start_x..end_x {
            let px = x as f64 + 0.5; // Pixel center
            let py = y as f64 + 0.5;
            
            let mut inside = false;
            let mut j = vertices.len() - 1;
            for i in 0..vertices.len() {
                let (xi, yi) = vertices[i];
                let (xj, yj) = vertices[j];
                
                let intersect = ((yi > py) != (yj > py)) &&
                    (px < (xj - xi) * (py - yi) / (yj - yi) + xi);
                if intersect {
                    inside = !inside;
                }
                j = i;
            }
            
            if inside {
                grid[y * width + x] = 1.0;
            }
        }
    }
}
