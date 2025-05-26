use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Copy, Clone, Default)]
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov_y: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.position, &self.target, &self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect_ratio, self.fov_y, self.near, self.far)
    }

    pub fn view_proj_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
}
