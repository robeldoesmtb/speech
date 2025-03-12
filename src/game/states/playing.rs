// src/game/states/playing.rs
use crate::engine::state::GameState;
use crate::engine::graphics::Renderer;
use crate::game::entities::player::Player;
use crate::game::level::{World, Level, TileType, Perspective};
use winit::event::{WindowEvent, VirtualKeyCode, ElementState, KeyboardInput};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use std::path::Path;
use std::fs;

pub struct PlayingState {
    player: Player,
    renderer: Renderer,
    world: World,
    camera_x: f32,
    camera_y: f32,
    assets_loaded: bool,
}

impl PlayingState {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let mut renderer = Renderer::new(device);
        let player = Player::new(100.0, 300.0);
        let mut world = World::new();
        
        // Create a test level for now
        let test_level_data = "
####################
#                  #
#                  #
#     E            #
#   #####          #
#                  #
#         S        #
#                  #
#                  #
#       #####      #
#                  #
#                  #
#                 E#
####################
";
        let test_level = Level::from_string(test_level_data, Perspective::SideScrolling);
        world.add_level("test_level", test_level);
        
        // Create another level with a top-down perspective
        let topdown_level_data = "
####################
#                  #
#     E            #
#                  #
#   #####          #
#                  #
#         S        #
#        ###       #
#         #        #
#       #####      #
#                  #
#            E     #
#                  #
####################
";
        let topdown_level = Level::from_string(topdown_level_data, Perspective::TopDown);
        world.add_level("topdown_level", topdown_level);
        
        let mut state = Self {
            player,
            renderer,
            world,
            camera_x: 0.0,
            camera_y: 0.0,
            assets_loaded: false,
        };
        
        // Initialize the player position based on the level's spawn point
        if let Some(level) = state.world.current_level() {
            state.player.x = level.spawn_point.0;
            state.player.y = level.spawn_point.1;
        }
        
        state
    }
    
    pub fn new_empty() -> Self {
        Self {
            player: Player::new(0.0, 0.0),
            renderer: Renderer::new_empty(),
            world: World::new(),
            camera_x: 0.0,
            camera_y: 0.0,
            assets_loaded: false,
        }
    }
    
    // Load game assets
    pub fn load_assets(&mut self, device: &Device, queue: &Queue) {
        if self.assets_loaded {
            return;
        }
        
        // Placeholder for loading player sprite
        // In a real game, you'd load texture files from disk
        let player_sprite_bytes = include_bytes!("../../../assets/player.png");
        self.renderer.load_texture(device, queue, "player", player_sprite_bytes)
            .expect("Failed to load player texture");
        
        // Load tile textures
        let platform_sprite_bytes = include_bytes!("../../../assets/platform.png");
        self.renderer.load_texture(device, queue, "platform", platform_sprite_bytes)
            .expect("Failed to load platform texture");
        
        let evidence_sprite_bytes = include_bytes!("../../../assets/evidence.png");
        self.renderer.load_texture(device, queue, "evidence", evidence_sprite_bytes)
            .expect("Failed to load evidence texture");
        
        self.assets_loaded = true;
    }
    
    // Update camera position to follow the player
    fn update_camera(&mut self, screen_width: f32, screen_height: f32) {
        // Target position is the player
        let target_x = self.player.x - screen_width / 2.0;
        let target_y = self.player.y - screen_height / 2.0;
        
        // Smoothly move the camera towards the target
        self.camera_x += (target_x - self.camera_x) * 0.1;
        self.camera_y += (target_y - self.camera_y) * 0.1;
        
        // Ensure the camera doesn't go outside the level boundaries
        if let Some(level) = self.world.current_level() {
            let level_width = level.width as f32 * 32.0; // 32 pixels per tile
            let level_height = level.height as f32 * 32.0;
            
            if self.camera_x < 0.0 {
                self.camera_x = 0.0;
            } else if self.camera_x > level_width - screen_width {
                self.camera_x = level_width - screen_width;
            }
            
            if self.camera_y < 0.0 {
                self.camera_y = 0.0;
            } else if self.camera_y > level_height - screen_height {
                self.camera_y = level_height - screen_height;
            }
        }
    }
}

impl GameState for PlayingState {
    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    state, 
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                match (keycode, state) {
                    (VirtualKeyCode::Escape, ElementState::Pressed) => {
                        // Exit on Escape
                        return true;
                    },
                    (VirtualKeyCode::Left, ElementState::Pressed) => {
                        self.player.move_left(true);
                    },
                    (VirtualKeyCode::Right, ElementState::Pressed) => {
                        self.player.move_right(true);
                    },
                    (VirtualKeyCode::Left, ElementState::Released) => {
                        self.player.move_left(false);
                    },
                    (VirtualKeyCode::Right, ElementState::Released) => {
                        self.player.move_right(false);
                    },
                    (VirtualKeyCode::Up, ElementState::Pressed) => {
                        // In top-down mode, move up; in side-scrolling mode, jump
                        if let Some(level) = self.world.current_level() {
                            match level.perspective {
                                Perspective::SideScrolling => self.player.jump(),
                                Perspective::TopDown => self.player.move_up(true),
                            }
                        }
                    },
                    (VirtualKeyCode::Up, ElementState::Released) => {
                        self.player.move_up(false);
                    },
                    (VirtualKeyCode::Down, ElementState::Pressed) => {
                        self.player.move_down(true);
                    },
                    (VirtualKeyCode::Down, ElementState::Released) => {
                        self.player.move_down(false);
                    },
                    (VirtualKeyCode::Space, ElementState::Pressed) => {
                        self.player.jump(); // Jump is also bound to space
                    },
                    (VirtualKeyCode::Tab, ElementState::Pressed) => {
                        // Switch perspective/level on Tab
                        if self.world.current_level == "test_level" {
                            self.world.switch_level("topdown_level");
                        } else {
                            self.world.switch_level("test_level");
                        }
                        
                        // Reset player position to the level's spawn point
                        if let Some(level) = self.world.current_level() {
                            self.player.x = level.spawn_point.0;
                            self.player.y = level.spawn_point.1;
                        }
                    },
                    _ => {}
                }
                // Returning false means we've handled the event
                false
            }
            _ => false,
        }
    }
    
    fn update(&mut self, dt: f32) {
        // Update player position and state
        if let Some(level) = self.world.current_level() {
            self.player.update(dt, level);
        }
        
        // Update camera
        self.update_camera(800.0, 600.0);  // Assuming screen size
    }
    
    fn render(&mut self, device: &Device, queue: &Queue, surface: &Surface, 
              config: &SurfaceConfiguration) -> Result<(), wgpu::SurfaceError> {
        // Ensure assets are loaded
        self.load_assets(device, queue);
        
        // Get a new frame
        let frame = self.renderer.begin_frame(surface)?;
        
        // Clear the screen with a nice background color
        let view = self.renderer.clear_screen(&frame, device, queue, wgpu::Color {
            r: 0.4,
            g: 0.6,
            b: 0.9,
            a: 1.0,
        });
        
        // Render the level
        if let Some(level) = self.world.current_level() {
            for y in 0..level.height {
                for x in 0..level.width {
                    if let Some(tile) = level.get_tile(x, y) {
                        match tile {
                            TileType::Platform => {
                                // Draw a platform tile
                                self.renderer.draw_sprite(
                                    device,
                                    queue,
                                    &view,
                                    "platform",
                                    (x as f32 * 32.0) - self.camera_x,
                                    (y as f32 * 32.0) - self.camera_y,
                                    32.0,
                                    32.0
                                );
                            },
                            TileType::Wall => {
                                // Draw a wall tile
                                self.renderer.draw_sprite(
                                    device,
                                    queue,
                                    &view,
                                    "platform",  // Using the same texture for now
                                    (x as f32 * 32.0) - self.camera_x,
                                    (y as f32 * 32.0) - self.camera_y,
                                    32.0,
                                    32.0
                                );
                            },
                            TileType::Evidence => {
                                // Check if this evidence has been collected
                                let evidence_id = format!("evidence_{}_{}", x, y);
                                if !self.player.evidence_collected.contains(&evidence_id) {
                                    // Draw evidence only if not collected
                                    self.renderer.draw_sprite(
                                        device,
                                        queue,
                                        &view,
                                        "evidence",
                                        (x as f32 * 32.0) - self.camera_x,
                                        (y as f32 * 32.0) - self.camera_y,
                                        32.0,
                                        32.0
                                    );
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Render the player
        let player_sprite = if self.player.facing_right { "player" } else { "player" }; // We'll add flipped sprites later
        self.renderer.draw_sprite(
            device,
            queue,
            &view,
            player_sprite,
            self.player.x - self.camera_x,
            self.player.y - self.camera_y,
            self.player.width,
            self.player.height
        );
        
        // Present the frame
        self.renderer.end_frame(frame);
        
        Ok(())
    }
}