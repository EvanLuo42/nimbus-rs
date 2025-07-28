use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, EuclideanSpace, Matrix4, Point3, Vector3};

#[derive(Copy, Clone)]
pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            eye: Point3::origin(),
            target: Point3::origin(),
            up: Vector3::unit_x(),
            aspect: 16.0 / 9.0,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0
        }
    }
}

impl Camera {
    pub fn projection(&self) -> Matrix4<f32> {
        perspective(Deg(self.fov_y), self.aspect, self.z_near, self.z_far)
    }
    
    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view: Matrix4::identity().into(),
            projection: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view = camera.view().into();
        self.projection = camera.projection().into();
    }
}
