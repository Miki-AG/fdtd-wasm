use fdtd_wasm::rasterizer::{parse_svg_path, fill_path_on_grid, rasterize_path, rasterize_obstacles, PathCommand};

#[test]
fn test_parse_svg_path_simple_rect() {
    let path = "M 0 0 L 10 0 L 10 10 L 0 10 Z";
    let commands = parse_svg_path(path).expect("Failed to parse valid path");
    
    assert!(!commands.is_empty());
    
    match commands[0] {
        PathCommand::MoveTo { x, y } => {
            assert_eq!(x, 0.0);
            assert_eq!(y, 0.0);
        },
        _ => panic!("First command should be MoveTo"),
    }
}

#[test]
fn test_parse_invalid_path() {
    // Garbage input
    let res = parse_svg_path("Not a path");
    assert!(res.is_err());
    
    // Missing coordinates
    let res2 = parse_svg_path("M 10"); 
    assert!(res2.is_err());
}

#[test]
fn test_fill_path_on_grid_square() {
    let width = 10;
    let height = 10;
    let mut grid = vec![0.0; width * height];
    
    let commands = vec![
        PathCommand::MoveTo { x: 3.0, y: 3.0 },
        PathCommand::LineTo { x: 7.0, y: 3.0 },
        PathCommand::LineTo { x: 7.0, y: 7.0 },
        PathCommand::LineTo { x: 3.0, y: 7.0 },
        PathCommand::ClosePath,
    ];
    
    fill_path_on_grid(width, height, &commands, &mut grid);
    
    // Check center is filled (1.0)
    assert_eq!(grid[5 * width + 5], 1.0);
    // Check corner is empty (0.0)
    assert_eq!(grid[0], 0.0);
}

#[test]
fn test_fill_path_out_of_bounds() {
    let width = 10;
    let height = 10;
    let mut grid = vec![0.0; width * height];
    
    // Commands for a square entirely outside (20,20)
    let commands = vec![
        PathCommand::MoveTo { x: 20.0, y: 20.0 },
        PathCommand::LineTo { x: 30.0, y: 20.0 },
        PathCommand::LineTo { x: 30.0, y: 30.0 },
        PathCommand::LineTo { x: 20.0, y: 30.0 },
        PathCommand::ClosePath,
    ];
    
    // Should not panic
    fill_path_on_grid(width, height, &commands, &mut grid);
    
    // Grid should remain empty
    assert!(grid.iter().all(|&x| x == 0.0));
}

#[test]
fn test_rasterize_path_delegates_correctly() {
    let width = 10;
    let height = 10;
    let mut grid = vec![0.0; width * height];
    let path = "M 3 3 L 7 3 L 7 7 L 3 7 Z";
    
    rasterize_path(width, height, path, &mut grid);
    
    assert_eq!(grid[5 * width + 5], 1.0);
}

#[test]
fn test_rasterize_obstacles_multiple_paths() {
    let width = 10;
    let height = 10;
    let paths = vec![
        "M 1 1 L 2 1 L 2 2 L 1 2 Z".to_string(),
        "M 8 8 L 9 8 L 9 9 L 8 9 Z".to_string(),
    ];
    
    let mask = rasterize_obstacles(width, height, &paths);
    
    assert_eq!(mask[1 * width + 1], 1.0);
    assert_eq!(mask[8 * width + 8], 1.0);
    assert_eq!(mask[5 * width + 5], 0.0);
}