use glium::{
    DrawError, DrawParameters, Program, Surface, VertexBuffer,
    backend::Facade,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    texture::{RawImage2d, Texture2d},
    uniform,
};
use image::{GenericImageView, ImageError};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct ImageRenderer<'a> {
    program: Program,
    quad_vbo: VertexBuffer<Vertex>,
    indices: NoIndices,
    draw_params: DrawParameters<'a>,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    texture: Option<Texture2d>,
    transform: [[f32; 4]; 4],
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub struct ImageRendererBuilder<'a, F: Facade> {
    display: &'a F,
    vertex_shader: Option<&'a str>,
    fragment_shader: Option<&'a str>,
    position: [f32; 2],
    size: [f32; 2],
    rotation: f32,
    color: [f32; 4],
    texture_bytes: Option<&'a [u8]>,
}

impl<'a, F: Facade> ImageRendererBuilder<'a, F> {
    pub fn new(display: &'a F) -> Self {
        Self {
            display,
            vertex_shader: None,
            fragment_shader: None,
            position: [0.0, 0.0],
            size: [100.0, 100.0],
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            texture_bytes: None,
        }
    }

    pub fn with_shaders(mut self, vs: &'a str, fs: &'a str) -> Self {
        self.vertex_shader = Some(vs);
        self.fragment_shader = Some(fs);
        self
    }

    pub fn with_size(mut self, size: [f32; 2]) -> Self {
        self.size = size;
        self
    }

    pub fn with_rotation(mut self, angle_rad: f32) -> Self {
        self.rotation = angle_rad;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_texture_bytes(mut self, bytes: &'a [u8]) -> Self {
        self.texture_bytes = Some(bytes);
        self
    }

    pub fn with_position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }

    fn calculate_transform(&self) -> [[f32; 4]; 4] {
        let px = 2.0 * self.position[0] / SCREEN_WIDTH as f32 - 1.0;
        let py = 1.0 - 2.0 * self.position[1] / SCREEN_HEIGHT as f32;
        let sx = 2.0 * self.size[0] / SCREEN_WIDTH as f32;
        let sy = 2.0 * self.size[1] / SCREEN_HEIGHT as f32;

        let (sin, cos) = self.rotation.sin_cos();

        [
            [sx * cos, -sy * sin, 0.0, 0.0],
            [sx * sin, sy * cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [px, py, 0.0, 1.0],
        ]
    }

    pub fn build(self) -> Result<ImageRenderer<'a>, BuildError> {
        let vertex_shader = self.vertex_shader.unwrap_or(DEFAULT_VERTEX_SHADER);
        let fragment_shader = self.fragment_shader.unwrap_or(DEFAULT_FRAGMENT_SHADER);

        let program = Program::from_source(self.display, vertex_shader, fragment_shader, None)
            .map_err(BuildError::ShaderCompilation)?;

        let quad = [
            Vertex {
                position: [-0.5, -0.5],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                tex_coords: [1.0, 1.0],
            },
        ];
        let quad_vbo =
            VertexBuffer::new(self.display, &quad).map_err(BuildError::BufferCreation)?;

        let indices = NoIndices(PrimitiveType::TriangleStrip);

        let texture = if let Some(bytes) = self.texture_bytes {
            Some(Self::load_texture_from_bytes(self.display, bytes)?)
        } else {
            None
        };

        Ok(ImageRenderer {
            program,
            quad_vbo,
            indices,
            draw_params: DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
            position: self.position,
            size: self.size,
            rotation: self.rotation,
            color: self.color,
            transform: self.calculate_transform(),
            texture,
        })
    }

    fn load_texture_from_bytes(display: &F, bytes: &[u8]) -> Result<Texture2d, BuildError> {
        let img = image::load_from_memory(bytes).map_err(BuildError::ImageLoad)?;
        let dims = img.dimensions();
        let raw = RawImage2d::from_raw_rgba_reversed(&img.to_rgba8().into_vec(), dims);
        Texture2d::new(display, raw).map_err(BuildError::TextureCreation)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(glium::ProgramCreationError),
    #[error("Buffer creation failed: {0}")]
    BufferCreation(glium::vertex::BufferCreationError),
    #[error("Image load failed: {0}")]
    ImageLoad(ImageError),
    #[error("Texture creation failed: {0}")]
    TextureCreation(glium::texture::TextureCreationError),
}

impl ImageRenderer<'_> {
    pub fn draw<S: Surface>(&self, target: &mut S) -> Result<(), DrawError> {
        let Some(texture) = self.texture.as_ref() else {
            return Err(DrawError::UnsupportedVerticesPerPatch); // Или кастомная ошибка
        };

        let uniforms = uniform! {
            tex: texture.sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            transform: self.transform,
            tint: self.color,
        };

        target.draw(
            &self.quad_vbo,
            self.indices,
            &self.program,
            &uniforms,
            &self.draw_params,
        )
    }
}

const DEFAULT_VERTEX_SHADER: &str = r#"
    #version 140
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;
    uniform mat4 transform;
    void main() {
        v_tex_coords = tex_coords;
        gl_Position = transform * vec4(position, 0.0, 1.0);
    }
"#;

const DEFAULT_FRAGMENT_SHADER: &str = r#"
    #version 140
    in vec2 v_tex_coords;
    out vec4 frag_color;
    uniform sampler2D tex;
    uniform vec4 tint;
    void main() {
        frag_color = texture(tex, v_tex_coords) * tint;
    }
"#;
