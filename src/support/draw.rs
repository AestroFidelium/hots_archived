use crate::support::*;
use cgmath::Rad;
use cgmath::SquareMatrix;
use cgmath::{Matrix4, Vector3};
use glium::Display;
use glutin::surface::WindowSurface;

pub const SCREEN_WIDTH: u32 = 1920;
pub const SCREEN_HEIGHT: u32 = 1080;

pub fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub fn create_box(
    display: &glium::Display<WindowSurface>,
) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
    let box_vertex_buffer = glium::VertexBuffer::new(
        display,
        &[
            Vertex {
                position: [0.5, -0.5, -0.5, 1.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5, 1.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 1.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 1.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5, 1.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 1.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5, 1.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5, 1.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 1.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 1.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 1.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5, 1.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5, 1.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5, 1.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5, 1.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5, 1.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5, 1.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5, 1.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 1.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5, 1.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5, 1.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5, 1.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 1.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 1.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
        ],
    )
    .unwrap();

    let mut indexes = Vec::new();
    for face in 0..6u16 {
        indexes.push(4 * face);
        indexes.push(4 * face + 1);
        indexes.push(4 * face + 2);
        indexes.push(4 * face);
        indexes.push(4 * face + 2);
        indexes.push(4 * face + 3);
    }
    let box_index_buffer = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &indexes,
    )
    .unwrap();
    (box_vertex_buffer, box_index_buffer)
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
}
implement_vertex!(Vertex, position, normal);

#[derive(Clone, Debug, Resource, Copy)]
pub struct ModelData {
    pub model_matrix: Matrix4<f32>,
    pub depth_mvp: Matrix4<f32>,
    pub color: [f32; 4],
}
impl ModelData {
    pub fn with_matrix(mut self, model: Matrix4<f32>) -> Self {
        self.model_matrix = model;
        self
    }

    pub fn color(c: [f32; 4]) -> Self {
        Self {
            model_matrix: cgmath::Matrix4::identity(),
            depth_mvp: cgmath::Matrix4::identity(),
            color: [c[0], c[1], c[2], c[3]],
        }
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.model_matrix = self.model_matrix * cgmath::Matrix4::from_scale(s);
        self
    }
    pub fn scale_xyz(mut self, sx: f32, sy: f32, sz: f32) -> Self {
        self.model_matrix = self.model_matrix * cgmath::Matrix4::from_nonuniform_scale(sx, sy, sz);
        self
    }

    pub fn translate(mut self, t: [f32; 3]) -> Self {
        self.model_matrix = self.model_matrix * cgmath::Matrix4::from_translation(t.into());
        self
    }
    pub fn rotate_pitch(mut self, angle_rad: f32) -> Self {
        self.model_matrix = self.model_matrix * Matrix4::from_angle_x(Rad(angle_rad));
        self
    }

    pub fn rotate_yaw(mut self, angle_rad: f32) -> Self {
        self.model_matrix = self.model_matrix * Matrix4::from_angle_y(Rad(angle_rad));
        self
    }

    pub fn rotate_roll(mut self, angle_rad: f32) -> Self {
        self.model_matrix = self.model_matrix * Matrix4::from_angle_z(Rad(angle_rad));
        self
    }
    pub fn rotate_axis(mut self, axis: Vector3<f32>, angle_rad: f32) -> Self {
        self.model_matrix = self.model_matrix * Matrix4::from_axis_angle(axis, Rad(angle_rad));
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DebugVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(DebugVertex, position, tex_coords);
impl DebugVertex {
    pub fn new(position: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            tex_coords,
        }
    }
}

pub fn load_wavefront(
    display: &Display<WindowSurface>,
    data: &[u8],
) -> glium::vertex::VertexBufferAny {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3],
        texture: [f32; 2],
    }

    implement_vertex!(Vertex, position, normal, texture);

    let mut data = ::std::io::BufReader::new(data);
    let data = obj::ObjData::load_buf(&mut data).unwrap();

    let mut vertex_data = Vec::new();

    for object in data.objects.iter() {
        for polygon in object.groups.iter().flat_map(|g| g.polys.iter()) {
            match polygon {
                obj::SimplePolygon(indices) => {
                    for v in indices.iter() {
                        let position = data.position[v.0];
                        let texture = v.1.map(|index| data.texture[index]);
                        let normal = v.2.map(|index| data.normal[index]);

                        let texture = texture.unwrap_or([0.0, 0.0]);
                        let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                        vertex_data.push(Vertex {
                            position,
                            normal,
                            texture,
                        })
                    }
                }
            }
        }
    }

    glium::vertex::VertexBuffer::new(display, &vertex_data)
        .unwrap()
        .into()
}

pub fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}
