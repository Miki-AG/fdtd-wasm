use fdtd_wasm::rasterizer::{parse_svg_path, fill_path_on_grid, PathCommand};

#[test]
fn test_parse_svg_path_simple_rect() {
    let path = "M 0 0 L 10 0 L 10 10 L 0 10 Z";
    let commands = parse_svg_path(path).expect("Failed to parse valid path");
    
    // We expect at least MoveTo, LineTo, ClosePath
    // Exact count depends on implementation, but basic check:
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
fn test_fill_path_on_grid_square() {
    let width = 10;
    let height = 10;
    let mut grid = vec![0.0; width * height];
    
    // Commands for a 4x4 square in the middle (3,3) to (7,7)
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
