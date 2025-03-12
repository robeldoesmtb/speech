use winit::event::WindowEvent;
use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

// GameState trait defines what all game states must implement
pub trait GameState {
    // Process window events like mouse moves, key presses, etc.
    fn handle_event(&mut self, event: &WindowEvent) -> bool;
    
    // Update game logic
    fn update(&mut self, dt: f32);
    
    // Render the current state
    fn render(&mut self, device: &Device, queue: &Queue, surface: &Surface, 
              config: &SurfaceConfiguration) -> Result<(), wgpu::SurfaceError>;
}

// StateManager holds our graphics resources and the current game state
pub struct StateManager {
    pub window: Window,
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    current_state: Box<dyn GameState>,
}

impl StateManager {
    // Create a new state manager with the given window and initial state
    pub fn new(window: Window, device: Device, queue: Queue, initial_state: Box<dyn GameState>) -> Self {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        let surface = unsafe { instance.create_surface(&window) }
            .expect("Failed to create surface");
        
        let adapter = futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).expect("Failed to find an appropriate adapter");
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        
        surface.configure(&device, &config);
        
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            current_state: initial_state,
        }
    }
    
    // Handle window events and pass them to the current state
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
                false
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
                false
            },
            // Let the current state handle other events
            _ => self.current_state.handle_event(event),
        }
    }
    
    // Handle window resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    // Update the current state
    pub fn update(&mut self, dt: f32) {
        self.current_state.update(dt);
    }
    
    // Render the current state
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.current_state.render(&self.device, &self.queue, &self.surface, &self.config)
    }
    
    // Switch to a new state
    pub fn change_state(&mut self, new_state: Box<dyn GameState>) {
        self.current_state = new_state;
    }
}