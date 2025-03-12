mod engine;
mod game;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use engine::state::StateManager;
use engine::graphics::Timer;
use game::states::playing::PlayingState;

fn main() {
    // Initialize the event loop
    let event_loop = EventLoop::new();
    
    // Create a window
    let window = WindowBuilder::new()
        .with_title("Speech")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window");
    
    // This is a bit complex because of Rust's ownership model
    let mut state_manager = {
        // Create initial resources
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
        
        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )).expect("Failed to create device");
        
        // Create our proper playing state with the device
        let playing_state = Box::new(PlayingState::new(&device, &queue));
        
        // Create the state manager
        StateManager::new(window, device, queue, playing_state)
    };
    
    // Create a timer for calculating delta time
    let mut timer = Timer::new();
    
    // Start the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::WindowEvent { 
                event, 
                window_id 
            } if window_id == state_manager.window.id() => {
                // Check if our state manager wants to exit
                if state_manager.handle_window_event(&event) {
                    println!("Window close requested!");
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::MainEventsCleared => {
                // Calculate delta time
                let dt = timer.delta();
                
                // Update game state
                state_manager.update(dt);
                
                // Request to redraw the window
                state_manager.window.request_redraw();
            },
            Event::RedrawRequested(window_id) if window_id == state_manager.window.id() => {
                // Render the current state
                match state_manager.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state_manager.resize(state_manager.size),
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            _ => (),
        }
    });
}