// src/game/entities/player.rs
use crate::game::level::{Level, TileType, Perspective};


const ACCELERATION: f32 = 1000.0;     // How quickly the player accelerates
const MAX_VELOCITY: f32 = 500.0;      // Maximum running speed
const FRICTION: f32 = 800.0;          // How quickly the player slows down
const JUMP_VELOCITY: f32 = 500.0;     // Initial upward velocity when jumping
const GRAVITY: f32 = 1500.0;          // Downward acceleration
const TILE_SIZE: f32 = 32.0;          // Size of each tile

pub struct Player {
    // Position
    pub x: f32,
    pub y: f32,
    
    // Velocity
    pub velocity_x: f32,
    pub velocity_y: f32,
    
    // Movement state
    pub moving_left: bool,
    pub moving_right: bool,
    pub moving_up: bool,
    pub moving_down: bool,
    pub is_jumping: bool,
    pub is_grounded: bool,
    
    // Characteristics
    pub width: f32,
    pub height: f32,
    
    // For animation
    pub facing_right: bool,
    pub animation_frame: usize,
    pub animation_timer: f32,
    
    // Evidence collected
    pub evidence_collected: Vec<String>,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            velocity_x: 0.0,
            velocity_y: 0.0,
            moving_left: false,
            moving_right: false,
            moving_up: false,
            moving_down: false,
            is_jumping: false,
            is_grounded: true,
            width: 24.0,  // Slightly smaller than a tile
            height: 48.0, // Taller than a tile
            facing_right: true,
            animation_frame: 0,
            animation_timer: 0.0,
            evidence_collected: Vec::new(),
        }
    }
    
    // Handle movement input
    pub fn move_left(&mut self, pressed: bool) {
        self.moving_left = pressed;
        if pressed {
            self.facing_right = false;
        }
    }
    
    pub fn move_right(&mut self, pressed: bool) {
        self.moving_right = pressed;
        if pressed {
            self.facing_right = true;
        }
    }
    
    pub fn move_up(&mut self, pressed: bool) {
        self.moving_up = pressed;
    }
    
    pub fn move_down(&mut self, pressed: bool) {
        self.moving_down = pressed;
    }
    
    pub fn jump(&mut self) {
        if self.is_grounded {
            self.velocity_y = -JUMP_VELOCITY; // Negative is up in screen coordinates
            self.is_jumping = true;
            self.is_grounded = false;
        }
    }
    
    // Update player position and physics
    pub fn update(&mut self, dt: f32, level: &Level) {
        match level.perspective {
            Perspective::SideScrolling => self.update_side_scrolling(dt, level),
            Perspective::TopDown => self.update_top_down(dt, level),
        }
        
        // Update animation
        self.animation_timer += dt;
        if self.animation_timer > 0.1 {  // Change frame every 0.1 seconds
            self.animation_timer = 0.0;
            self.animation_frame = (self.animation_frame + 1) % 4;  // 4 frames of animation
        }
        
        // Check for evidence collection
        self.check_evidence_collection(level);
    }
    
    // Update in side-scrolling mode
    fn update_side_scrolling(&mut self, dt: f32, level: &Level) {
        // Apply horizontal movement based on input
        if self.moving_left {
            self.velocity_x -= ACCELERATION * dt;
        }
        
        if self.moving_right {
            self.velocity_x += ACCELERATION * dt;
        }
        
        // Apply friction when not moving
        if !self.moving_left && !self.moving_right && self.is_grounded {
            // Slow down gradually
            if self.velocity_x > 0.0 {
                self.velocity_x -= FRICTION * dt;
                if self.velocity_x < 0.0 {
                    self.velocity_x = 0.0;
                }
            } else if self.velocity_x < 0.0 {
                self.velocity_x += FRICTION * dt;
                if self.velocity_x > 0.0 {
                    self.velocity_x = 0.0;
                }
            }
        }
        
        // Apply gravity
        if !self.is_grounded {
            self.velocity_y += GRAVITY * dt;
        }
        
        // Cap horizontal velocity
        if self.velocity_x > MAX_VELOCITY {
            self.velocity_x = MAX_VELOCITY;
        } else if self.velocity_x < -MAX_VELOCITY {
            self.velocity_x = -MAX_VELOCITY;
        }
        
        // Store original position for collision detection
        let original_x = self.x;
        let original_y = self.y;
        
        // Update position
        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;
        
        // Check for collisions with the level
        self.handle_collisions(level, original_x, original_y);
    }
    
    // Update in top-down mode
    fn update_top_down(&mut self, dt: f32, level: &Level) {
        // In top-down mode, we use a simpler movement model
        let mut dx = 0.0;
        let mut dy = 0.0;
        
        if self.moving_left {
            dx -= MAX_VELOCITY;
            self.facing_right = false;
        }
        
        if self.moving_right {
            dx += MAX_VELOCITY;
            self.facing_right = true;
        }
        
        if self.moving_up {
            dy -= MAX_VELOCITY;
        }
        
        if self.moving_down {
            dy += MAX_VELOCITY;
        }
        
        // Normalize diagonal movement
        if dx != 0.0 && dy != 0.0 {
            let magnitude = (dx * dx + dy * dy).sqrt();
            dx = dx / magnitude * MAX_VELOCITY;
            dy = dy / magnitude * MAX_VELOCITY;
        }
        
        // Store original position for collision detection
        let original_x = self.x;
        let original_y = self.y;
        
        // Update position
        self.x += dx * dt;
        self.y += dy * dt;
        
        // Check for collisions with the level
        self.handle_collisions(level, original_x, original_y);
    }
    
    // Handle collisions with the level
fn handle_collisions(&mut self, level: &Level, original_x: f32, original_y: f32) {
    // Player's bounding box
    let left = self.x - self.width / 2.0;
    let right = self.x + self.width / 2.0;
    let top = self.y - self.height / 2.0;
    let bottom = self.y + self.height / 2.0;
    
    // Convert to tile coordinates
    let tile_left = (left / TILE_SIZE).floor() as usize;
    let tile_right = (right / TILE_SIZE).floor() as usize;
    let tile_top = (top / TILE_SIZE).floor() as usize;
    let tile_bottom = (bottom / TILE_SIZE).floor() as usize;
    
    // Check for horizontal collisions
    let mut collision_x = false;
    for y in tile_top..=tile_bottom {
        for x in tile_left..=tile_right {
            if let Some(tile) = level.get_tile(x, y) {
                match tile {
                    TileType::Platform | TileType::Wall => {
                        // If we were moving right and hit a wall
                        if self.velocity_x > 0.0 && right > x as f32 * TILE_SIZE {
                            self.x = x as f32 * TILE_SIZE - self.width / 2.0;
                            self.velocity_x = 0.0;
                            collision_x = true;
                        }
                        // If we were moving left and hit a wall
                        else if self.velocity_x < 0.0 && left < (x as f32 + 1.0) * TILE_SIZE {
                            self.x = (x as f32 + 1.0) * TILE_SIZE + self.width / 2.0;
                            self.velocity_x = 0.0;
                            collision_x = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // If we didn't collide horizontally, restore the original x position
    if !collision_x {
        self.x = original_x;
    }
    
    // Update the bounding box after horizontal movement
    let left = self.x - self.width / 2.0;
    let right = self.x + self.width / 2.0;
    let top = self.y - self.height / 2.0;
    let bottom = self.y + self.height / 2.0;
    
    let tile_left = (left / TILE_SIZE).floor() as usize;
    let tile_right = (right / TILE_SIZE).floor() as usize;
    let tile_top = (top / TILE_SIZE).floor() as usize;
    let tile_bottom = (bottom / TILE_SIZE).floor() as usize;
    
    // Check for vertical collisions
    let mut collision_y = false;
    self.is_grounded = false; // Assume we're not grounded until proven otherwise
    
    for y in tile_top..=tile_bottom {
        for x in tile_left..=tile_right {
            if let Some(tile) = level.get_tile(x, y) {
                match tile {
                    TileType::Platform | TileType::Wall => {
                        // If we were moving down and hit a platform
                        if self.velocity_y > 0.0 && bottom > y as f32 * TILE_SIZE {
                            self.y = y as f32 * TILE_SIZE - self.height / 2.0;
                            self.velocity_y = 0.0;
                            self.is_grounded = true;
                            self.is_jumping = false;
                            collision_y = true;
                        }
                        // If we were moving up and hit a ceiling
                        else if self.velocity_y < 0.0 && top < (y as f32 + 1.0) * TILE_SIZE {
                            self.y = (y as f32 + 1.0) * TILE_SIZE + self.height / 2.0;
                            self.velocity_y = 0.0;
                            collision_y = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // If we didn't collide vertically, restore the original y position
    if !collision_y {
        self.y = original_y;
    }
    
    // Handle level boundaries
    if self.x < self.width / 2.0 {
        self.x = self.width / 2.0;
        self.velocity_x = 0.0;
    } else if self.x > level.width as f32 * TILE_SIZE - self.width / 2.0 {
        self.x = level.width as f32 * TILE_SIZE - self.width / 2.0;
        self.velocity_x = 0.0;
    }
    
    if self.y < self.height / 2.0 {
        self.y = self.height / 2.0;
        self.velocity_y = 0.0;
    } else if self.y > level.height as f32 * TILE_SIZE - self.height / 2.0 {
        self.y = level.height as f32 * TILE_SIZE - self.height / 2.0;
        self.velocity_y = 0.0;
        self.is_grounded = true;
        self.is_jumping = false;
    }
}

// Check if the player has collected any evidence
fn check_evidence_collection(&mut self, level: &Level) {
    // Player's bounding box
    let left = self.x - self.width / 2.0;
    let right = self.x + self.width / 2.0;
    let top = self.y - self.height / 2.0;
    let bottom = self.y + self.height / 2.0;
    
    // Convert to tile coordinates
    let tile_left = (left / TILE_SIZE).floor() as usize;
    let tile_right = (right / TILE_SIZE).floor() as usize;
    let tile_top = (top / TILE_SIZE).floor() as usize;
    let tile_bottom = (bottom / TILE_SIZE).floor() as usize;
    
    // Check for evidence tiles
    for y in tile_top..=tile_bottom {
        for x in tile_left..=tile_right {
            if let Some(tile) = level.get_tile(x, y) {
                match tile {
                    TileType::Evidence => {
                        let evidence_id = format!("evidence_{}_{}", x, y);
                        if !self.evidence_collected.contains(&evidence_id) {
                            self.evidence_collected.push(evidence_id);
                            println!("Evidence collected! Total: {}", self.evidence_collected.len());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}