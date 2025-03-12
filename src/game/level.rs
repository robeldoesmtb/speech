use std::collections::HashMap;

// Define different tile types
pub enum TileType {
    Empty,
    Platform,
    Wall,
    Evidence,
}

// A simple 2D tile-based level
pub struct Level {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    pub perspective: Perspective,
    pub spawn_point: (f32, f32),
    pub evidence_locations: Vec<(usize, usize)>,
}

// The perspective of the level
pub enum Perspective {
    SideScrolling,
    TopDown,
}

impl Level {
    // Create a new empty level
    pub fn new(width: usize, height: usize, perspective: Perspective) -> Self {
        let tiles = vec![TileType::Empty; width * height];
        Self {
            width,
            height,
            tiles,
            perspective,
            spawn_point: (0.0, 0.0),
            evidence_locations: Vec::new(),
        }
    }
    
    // Get a tile at a specific position
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&TileType> {
        if x < self.width && y < self.height {
            Some(&self.tiles[y * self.width + x])
        } else {
            None
        }
    }
    
    // Set a tile at a specific position
    pub fn set_tile(&mut self, x: usize, y: usize, tile_type: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile_type;
        }
    }
    
    // Set the spawn point
    pub fn set_spawn_point(&mut self, x: f32, y: f32) {
        self.spawn_point = (x, y);
    }
    
    // Add an evidence location
    pub fn add_evidence(&mut self, x: usize, y: usize) {
        self.evidence_locations.push((x, y));
        // Also update the tile to be evidence
        self.set_tile(x, y, TileType::Evidence);
    }
    
    // Load a level from a string representation
    pub fn from_string(data: &str, perspective: Perspective) -> Self {
        let lines: Vec<&str> = data.trim().lines().collect();
        let height = lines.len();
        let width = lines[0].len();
        
        let mut level = Self::new(width, height, perspective);
        
        for (y, line) in lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' => level.set_tile(x, y, TileType::Platform),
                    'W' => level.set_tile(x, y, TileType::Wall),
                    'E' => level.add_evidence(x, y),
                    'S' => {
                        level.set_spawn_point(x as f32 * 32.0, y as f32 * 32.0); // Assuming 32x32 tiles
                        level.set_tile(x, y, TileType::Empty);
                    },
                    _ => level.set_tile(x, y, TileType::Empty),
                }
            }
        }
        
        level
    }
}

// A collection of levels
pub struct World {
    pub levels: HashMap<String, Level>,
    pub current_level: String,
}

impl World {
    pub fn new() -> Self {
        Self {
            levels: HashMap::new(),
            current_level: String::new(),
        }
    }
    
    // Add a level to the world
    pub fn add_level(&mut self, name: &str, level: Level) {
        self.levels.insert(name.to_string(), level);
        if self.current_level.is_empty() {
            self.current_level = name.to_string();
        }
    }
    
    // Switch to a different level
    pub fn switch_level(&mut self, name: &str) -> bool {
        if self.levels.contains_key(name) {
            self.current_level = name.to_string();
            true
        } else {
            false
        }
    }
    
    // Get the current level
    pub fn current_level(&self) -> Option<&Level> {
        self.levels.get(&self.current_level)
    }
    
    // Get a mutable reference to the current level
    pub fn current_level_mut(&mut self) -> Option<&mut Level> {
        self.levels.get_mut(&self.current_level)
    }
}