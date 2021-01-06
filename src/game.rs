use std::time::{Instant, Duration};
use winit::{
    event::*,
};

use crate::camera::{Camera, CameraController};

pub struct GameState {
    pub last_frame: Instant,
    pub time_delta: Option<Duration>,
    pub last_cursor: Option<(u32, u32)>,
    pub current_sprite_frame: u32,
    pub sprite_frame_count: u32,
    pub last_sprite_frame_time: Instant,
    pub camera_controller: CameraController,
    pub camera: Camera,
}

impl GameState {
    pub fn new () -> GameState {
        let camera = Camera {
            center: cgmath::Vector2::new(0.0, 0.0),
            height: 3.0,
            aspect: 16.0/9.0,
            znear: -1.0,
            zfar: 100.0,
        };

        GameState {
            last_frame: Instant::now(),
            time_delta: Some(Instant::now().elapsed()),
            last_cursor: Some((0, 0)),
            current_sprite_frame: 0,
            sprite_frame_count: 24,
            last_sprite_frame_time: Instant::now(),
            camera_controller: CameraController::new(0.2),
            camera,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) {
        self.camera_controller.process_events(event);
    }

    pub fn update(&mut self) {
        let new_frame = Instant::now();
        let dt = new_frame - self.last_frame;
        self.time_delta = Some(dt);
        self.last_frame = new_frame;

        if dt.as_millis() > 0 {
            self.camera_controller.update_camera(&mut self.camera);

            if self.last_sprite_frame_time.elapsed().as_millis() > 60 {
                self.current_sprite_frame = (self.current_sprite_frame + 1) % self.sprite_frame_count;
                self.last_sprite_frame_time = Instant::now();
            }
        }
    }
}