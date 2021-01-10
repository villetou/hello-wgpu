extern crate imgui_winit_support;

mod rendering;
mod game;
mod texture;
mod camera;
mod controller;

use crate::rendering::State;
use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
};

use game::*;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut game = game::GameState::new();

    // Since main can't be async, we're going to need to block
    let mut state = futures::executor::block_on(State::new(&window, &game));
    

    event_loop.run(move |event, _, control_flow| {

        //*control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(5));
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) && !game.input(event) {
                    match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    },
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { 
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    },
                    _ => {}
                }
            },
            Event::RedrawEventsCleared => {
                
            }
            Event::RedrawRequested(_) => {
                match state.render(&game, &window) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.

                if game.last_frame.elapsed() > std::time::Duration::from_millis(10) {
                    game.update();
                    state.update(&game);
                    window.request_redraw();
                }
            },
            _ => {}
        }

        state.imgui.platform.handle_event(state.imgui.ctx.io_mut(), &window, &event);
    });
}