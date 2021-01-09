use std::time::{Instant, Duration};
use winit::{
    event::*,
};

use crate::camera::{Camera, CameraController};
use crate::controller::Controller;

pub struct Animator<'a> {
    current_frame: usize,
    current_frame_index: usize,
    last_frame_time: std::time::Instant,
    animation: &'a Animation
}

impl<'a> Animator<'_> {
    pub fn new(animation: &'a Animation) -> Animator<'a> {
        Animator {
            current_frame_index: 0,
            current_frame: animation.frames[0],
            last_frame_time: std::time::Instant::now(),
            animation,
        }
    }

    pub fn update(&mut self) -> Option<usize> {
        if self.last_frame_time.elapsed() > self.animation.default_timing {
            self.current_frame_index = (self.current_frame_index + 1) % self.animation.frames.len();
            self.current_frame = self.animation.frames[self.current_frame_index];
            self.last_frame_time = Instant::now();

            return Some(self.current_frame_index);
        }
        None
    }
}

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<usize>,
    pub default_timing: std::time::Duration,
}

pub enum Direction {
    N,
    //NE,
    E,
    //SE,
    S,
    //SW,
    W,
    //NW,
}

pub struct Instance<'a> {
    pub position: cgmath::Vector3<f32>,
    pub direction: Direction,
    pub frame: u32,
    pub animator: Animator<'a>
}

pub struct GameState<'a> {
    pub last_frame: Instant,
    pub time_delta: Option<Duration>,
    pub last_cursor: Option<(u32, u32)>,
    pub current_sprite_frame: u32,
    pub sprite_frame_count: u32,
    pub last_sprite_frame_time: Instant,
    pub camera_controller: CameraController,
    pub camera: Camera,
    pub instances: Vec<Instance<'a>>,
    pub controller: Controller,
    pub animations: Vec<Animation>,
}


impl<'a> GameState<'a> {
    pub fn new () -> GameState<'a> {
        let camera = Camera {
            center: cgmath::Vector2::new(0.0, 0.0),
            height: 3.0,
            aspect: 16.0/9.0,
            znear: -1.0,
            zfar: 100.0,
        };

        let mut instances: Vec<Instance> = Vec::<Instance>::new();
        
        let mut animations = Vec::new();

        animations.push(Animation { frames: (0..5).collect(), default_timing: std::time::Duration::from_millis(100) });
        animations.push(Animation { frames: (6..11).collect(), default_timing: std::time::Duration::from_millis(100) });
        animations.push(Animation { frames: (12..17).collect(), default_timing: std::time::Duration::from_millis(100) });
        animations.push(Animation { frames: (18..23).collect(), default_timing: std::time::Duration::from_millis(100) });

        instances.push(
            Instance {
                position: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                direction: Direction::N,
                frame: 0,
                animator: Animator::new(&animations[0])
            }
        );

        GameState {
            last_frame: Instant::now(),
            time_delta: Some(Instant::now().elapsed()),
            last_cursor: Some((0, 0)),
            current_sprite_frame: 0,
            sprite_frame_count: 24,
            last_sprite_frame_time: Instant::now(),
            camera_controller: CameraController::new(0.2),
            camera,
            instances,
            controller: Controller::new(0.2),
            animations,
        }
    }

    // TODO: Actually return true if an event was consumed
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.controller.process_events(event);
            match event {
                WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            
                        },
                        _ => {}
                    }
                },
                _ => {}
            }

        false
    }

    pub fn update(&mut self) {
        let new_frame = Instant::now();
        let dt = new_frame - self.last_frame;
        self.time_delta = Some(dt);
        self.last_frame = new_frame;

        if self.controller.is_down_pressed {
            self.instances[0].position[1] -= 0.05;
        }
        if self.controller.is_up_pressed {
            self.instances[0].position[1] += 0.05;
        }
        if self.controller.is_left_pressed {
            self.instances[0].position[0] -= 0.05;
        }
        if self.controller.is_right_pressed {
            self.instances[0].position[0] += 0.05;
        }
        if dt.as_millis() > 0 {
            self.camera_controller.update_camera(&mut self.camera);

        }
    }
}