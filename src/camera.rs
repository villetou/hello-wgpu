use winit::{
    event::*,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub center: cgmath::Vector2<f32>,
    pub aspect: f32,
    pub height: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::ortho(-self.height * self.aspect / 2.0 + self.center.x, self.height * self.aspect / 2.0 + self.center.x, self.center.y, self.height + self.center.y, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj;
    }
}

pub struct CameraController {}

impl CameraController {
    pub fn update_camera_with_controller(controller: &crate::controller::Controller, camera: &mut Camera) {
        if controller.is_up_pressed {
            camera.center.y = camera.center.y + controller.speed;
        }
        if controller.is_down_pressed {
            camera.center.y = camera.center.y - controller.speed;
        }
        if controller.is_right_pressed {
            camera.center.x = camera.center.x + controller.speed;
        }
        if controller.is_left_pressed {
            camera.center.x = camera.center.x - controller.speed;
        }
    }
}
