use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

use crate::ecs::{Camera, Keyboard, Mouse};
use crate::gltfmodel::{GltfModel, load_gltf_model_from_bytes};
use crate::image_renderer::ImageRendererBuilder;
use crate::support::*;
use cgmath::{Matrix4, Vector3, vec3};
use glium::winit::keyboard::PhysicalKey;
use glium::{Display, IndexBuffer, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::keyboard::{Key, KeyCode};

use winit::event::KeyEvent;

pub use bevy_ecs::prelude::*;
use winit::event::ElementState;

pub static GAME: LazyLock<Arc<RwLock<ClientInfo>>> =
    LazyLock::new(|| Arc::new(RwLock::new(ClientInfo::new())));

pub struct Application {
    pub shadow_texture: glium::texture::DepthTexture2d,
    pub model_vertex_buffer: VertexBuffer<Vertex>,
    pub model_index_buffer: IndexBuffer<u16>,
    pub shadow_map_shaders: Program,
    pub render_shaders: Program,
    pub shader_world: Program,
    pub shader_screen: Program,
    pub font_atlas: FontAtlas,
    pub camera: Camera,
    pub mouse: Mouse,
    pub keyboard: Keyboard,

    pub input_text: String,
    pub is_shift_held: bool,

    pub models: HashMap<String, GltfModel>,
}

impl Application {
    pub fn create_shader_world(display: &glium::Display<WindowSurface>) -> glium::Program {
        glium::Program::from_source(
            display,
            "
            #version 140
            in vec3 position;
            in vec2 tex_coords;
            out vec2 v_tex;
            uniform mat4 mvp;
            void main() {
                v_tex = tex_coords;
                gl_Position = mvp * vec4(position, 1.0);
            }
            ",
            "
            #version 140
            in vec2 v_tex;
            out vec4 fragColor;
            uniform sampler2D tex;
            uniform vec4 text_color;
            void main() {
                vec4 sampled = texture(tex, v_tex);
                
                fragColor = vec4(text_color.rgb, text_color.a * sampled.a);
            }
            ",
            None,
        )
        .unwrap()
    }
    pub fn create_sharer_screen(display: &glium::Display<WindowSurface>) -> glium::Program {
        glium::Program::from_source(
            display,
            "
            #version 330 core
            in vec2 position;
            in vec2 tex_coords;
            out vec2 v_tex_coords;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_tex_coords = tex_coords;
            }
            ",
            "
            #version 330 core
            in vec2 v_tex_coords;
            uniform sampler2D tex;
            uniform vec4 text_color;
            out vec4 color;
            void main() {
                vec4 sampled = texture(tex, v_tex_coords);
                
                float alpha = sampled.a * text_color.a;
                color = vec4(text_color.rgb, alpha);
                
                
            }
            ",
            None,
        )
        .unwrap()
    }
    pub fn create_shadow_map_shader(display: &glium::Display<WindowSurface>) -> glium::Program {
        glium::Program::from_source(
            display,
            "
                #version 330 core
                in vec4 position;
                uniform mat4 depth_mvp;
                void main() {
                    gl_Position = depth_mvp * position;
                }
            ",
            "
                #version 330 core
                layout(location = 0) out float fragmentdepth;
                void main(){
                    fragmentdepth = gl_FragCoord.z;
                }
            ",
            None,
        )
        .unwrap()
    }
    pub fn create_render_shaders(display: &glium::Display<WindowSurface>) -> glium::Program {
        glium::Program::from_source(
            display,
            "
                #version 330 core

                uniform mat4 mvp;
                uniform mat4 depth_bias_mvp;
                uniform mat4 model_matrix;
                uniform vec4 model_color;

                in vec4 position;
                in vec4 normal;

                out vec4 frag_pos;
                out vec4 shadow_coord;
                out vec4 model_normal;

                void main() {
                    vec4 world_pos = model_matrix * position;
                    gl_Position =  mvp * position;
                    frag_pos = world_pos;
                    model_normal = model_matrix * normal;
                    shadow_coord = depth_bias_mvp * position;
                }
            ",
            "
                #version 330 core

                uniform sampler2DShadow shadow_map;
                uniform vec3 light_loc;
                uniform vec3 view_pos;
                uniform vec4 model_color;
                uniform float shadow_map_size;

                in vec4 frag_pos;
                in vec4 shadow_coord;
                in vec4 model_normal;

                out vec4 color;

                void main() {
                    vec3 normal = normalize(model_normal.xyz);
                    vec3 light_dir = normalize(light_loc - frag_pos.xyz);
                    vec3 view_dir = normalize(view_pos - frag_pos.xyz);

                    
                    float ambient = 0.15;
                    float diff = max(dot(normal, light_dir), 0.0);

                    vec3 halfway = normalize(light_dir + view_dir);
                    float spec = pow(max(dot(normal, halfway), 0.0), 64.0);

                    
                    vec3 projCoords = shadow_coord.xyz / shadow_coord.w;
                    
                    if (projCoords.x < 0.0 || projCoords.x > 1.0 || projCoords.y < 0.0 || projCoords.y > 1.0 || projCoords.z > 1.0) {
                        float lighting = ambient + diff * 0.85 + spec * 0.6;
                        vec3 result = lighting * model_color.rgb;
                        
                        result = pow(result, vec3(1.0/2.2));
                        color = vec4(result, model_color.a);
                        return;
                    }

                    
                    float bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.0005);

                    
                    float visibility = 0.0;
                    float texelSize = 1.0 / shadow_map_size;
                    int samples = 0;
                    for (int x = -1; x <= 1; ++x) {
                        for (int y = -1; y <= 1; ++y) {
                            vec2 offset = vec2(float(x), float(y)) * texelSize;
                            
                            visibility += texture(shadow_map, vec3(projCoords.xy + offset, projCoords.z - bias));
                            samples += 1;
                        }
                    }
                    visibility = visibility / float(samples);

                    
                    float lighting = ambient + diff * visibility + spec * visibility;
                    vec3 result = lighting * model_color.rgb;

                    
                    result = pow(result, vec3(1.0/2.2));
                    color = vec4(result, model_color.a);
                }
            ",
            None,
        )
        .unwrap()
    }
    pub fn create_model_shaders(display: &glium::Display<WindowSurface>) -> glium::Program {
        glium::Program::from_source(
            display,
            // --- Vertex shader ---
            r#"
        #version 330 core

        uniform mat4 mvp;
        uniform mat4 model_matrix;
        uniform vec4 model_color;

        in vec3 position;
        in vec3 normal;
        in vec2 tex_coords;

        out vec3 frag_pos;
        out vec3 frag_normal;
        out vec2 v_tex_coords;

        void main() {
            vec4 world_pos = model_matrix * vec4(position, 1.0);
            gl_Position = mvp * vec4(position, 1.0);
            frag_pos = world_pos.xyz;
            frag_normal = mat3(model_matrix) * normal;
            v_tex_coords = tex_coords;
        }
        "#,
            // --- Fragment shader ---
            r#"
        #version 330 core

        uniform sampler2D diffuse_tex;   // обязательная текстура
        uniform vec3 light_loc;          // позиция света
        uniform vec3 view_pos;           // позиция камеры
        uniform vec4 model_color;        // цвет модели

        in vec3 frag_pos;
        in vec3 frag_normal;
        in vec2 v_tex_coords;

        out vec4 color;

        void main() {
            vec3 norm = normalize(frag_normal);
            vec3 light_dir = normalize(light_loc - frag_pos);
            float diff = max(dot(norm, light_dir), 0.0);

            vec4 tex_color = texture(diffuse_tex, v_tex_coords);
            vec3 diffuse = diff * tex_color.rgb * model_color.rgb;

            // простое затухание
            float ambient = 0.15;
            vec3 final_color = diffuse + ambient * tex_color.rgb * model_color.rgb;

            color = vec4(final_color, tex_color.a * model_color.a);
        }
        "#,
            None,
        )
        .unwrap()
    }

    fn handle_key_input(&mut self, event: &KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => {
                self.input_text.pop();
            }
            PhysicalKey::Code(KeyCode::Space) => {
                self.input_text.push(' ');
            }
            PhysicalKey::Code(KeyCode::Enter) => {
                self.input_text.clear();
            }
            PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => {
                self.is_shift_held = true;
            }
            _ => {
                if let Key::Character(ch) = &event.logical_key {
                    // если shift — делаем верхний регистр
                    let c = if self.is_shift_held {
                        ch.to_ascii_uppercase()
                    } else {
                        ch.to_ascii_lowercase()
                    };
                    self.input_text.push_str(&c);
                }
            }
        }
    }

    // В update снимаем флаг shift, когда отпущен
    pub fn update_shift_state(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) = event.physical_key {
            self.is_shift_held = false;
        }
    }
}

impl ApplicationContext for Application {
    const WINDOW_TITLE: &'static str = "Tracer Demo";

    fn new(display: &Display<WindowSurface>) -> Self {
        let shadow_map_size = 2048u32;

        let (model_vertex_buffer, model_index_buffer) = create_box(display);

        let shadow_map_shaders = Application::create_shadow_map_shader(display);
        let render_shaders = Application::create_render_shaders(display);

        let shadow_texture = glium::texture::DepthTexture2d::empty_with_format(
            display,
            glium::texture::DepthFormat::I32,
            glium::texture::MipmapsOption::NoMipmap,
            shadow_map_size,
            shadow_map_size,
        )
        .unwrap();

        let mut char_set: HashSet<char> = (32u8..127u8).map(|b| b as char).collect();
        char_set.extend('A'..='Z');
        char_set.extend('a'..='z');
        char_set.extend(('\u{0410}'..='\u{042F}').chain('\u{0430}'..='\u{044F}'));
        char_set.insert('ё');
        char_set.insert('Ё');
        char_set.extend(vec![' ', '.', ',', '!', '?', '-']);
        char_set.extend('0'..='9');
        let chars: Vec<char> = char_set.into_iter().collect();

        let atlas = FontAtlas::new(
            display,
            include_bytes!("../../assets/NotoSans-Bold.ttf"),
            64.0,
            &chars,
        );

        let mut models = HashMap::new();

        let mut tracer_model = load_gltf_model_from_bytes(
            display,
            include_bytes!("../../assets/tracer/tracer.glb"),
            include_bytes!("../../assets/tracer/storm_hero_tracer_base_diff.png"),
            Some(include_bytes!("../../assets/tracer/tracer_anims.glb")),
        )
        .unwrap();
        tracer_model.play_animation(6);

        // tracer_model.set_position(position);

        models.insert("TracerHero".to_string(), tracer_model);

        Self {
            shadow_texture,
            model_vertex_buffer,
            model_index_buffer,
            shadow_map_shaders,
            render_shaders,
            camera: Camera::new(),
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),

            shader_world: Application::create_shader_world(display),
            shader_screen: Application::create_sharer_screen(display),
            font_atlas: atlas,
            input_text: String::new(),
            is_shift_held: false,
            models,
        }
    }

    fn update(&mut self) {}

    #[allow(unused_variables)]
    fn draw_frame(&mut self, delta_time: f32, display: &Display<WindowSurface>) {
        let projection_matrix = self.camera.projection_matrix();

        let light_loc = {
            let x = 0.0;
            let z = 0.0;
            [x as f32, 20.0, z as f32]
        };

        {
            let w = 4.0;
            let depth_projection_matrix: cgmath::Matrix4<f32> =
                cgmath::ortho(-w, w, -w, w, -10.0, 20.0);
            let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
            let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
            let depth_view_matrix =
                cgmath::Matrix4::look_at_rh(light_loc.into(), view_center, view_up);

            let draw_params: glium::draw_parameters::DrawParameters<'_> = glium::DrawParameters {
                backface_culling: glium::BackfaceCullingMode::CullClockwise,

                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut target =
                glium::framebuffer::SimpleFrameBuffer::depth_only(display, &self.shadow_texture)
                    .unwrap();
            target.clear_color(1.0, 1.0, 1.0, 1.0);
            target.clear_depth(1.0);
        }

        let (width, height) = display.get_framebuffer_dimensions();
        let screen_ratio = width as f32 / height as f32;
        let perspective_matrix: cgmath::Matrix4<f32> =
            cgmath::perspective(cgmath::Deg(45.0), screen_ratio, 0.0001, 100.0);

        let view_matrix = self.camera.view_matrix();

        let bias_matrix: cgmath::Matrix4<f32> = [
            [0.5, 0.0, 0.0, 0.0],
            [0.0, 0.5, 0.0, 0.0],
            [0.0, 0.0, 0.5, 0.0],
            [0.5, 0.5, 0.5, 1.0],
        ]
        .into();

        let draw_params: glium::draw_parameters::DrawParameters<'_> = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            multisampling: true,
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let program = &self.render_shaders;
        let (width, height) = display.get_framebuffer_dimensions();
        let aspect = width as f32 / height as f32;
        let projection = self.camera.projection_matrix();

        let view: Matrix4<f32> = self.camera.view_matrix();

        let builder_global = Matrix4::from_translation(vec3(0.0, 0.0, 2.0))
            * Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0);

        // for models_update_animations in self.models.values_mut() {
        //     models_update_animations.update(delta_time);
        // }

        if let Ok(mut game) = GAME.try_write() {
            game.models.iter_mut().for_each(|model| {
                if let Some(gltf_model) = self.models.get_mut(&model.name) {
                    let mut draw_model = gltf_model.clone_model();
                    draw_model.play_animation(model.anim);
                    draw_model.set_position_xyz(model.pos[0], model.pos[1], model.pos[2]);
                    draw_model.set_rotation_xyz(model.rot[0], model.rot[1], model.rot[2]);
                    draw_model.update(delta_time);

                    draw_model
                        .draw_with_node_animation(
                            display,
                            &mut target,
                            &Application::create_model_shaders(display),
                            projection,
                            view,
                            vec3(0.0, 10.0, 10.0),
                            Vector3 {
                                x: self.camera.position.x,
                                y: self.camera.position.y,
                                z: self.camera.position.z,
                            },
                            &self.shadow_texture,
                            builder_global,
                            [1.0, 1.0, 1.0, 1.0],
                            2048.0f32,
                        )
                        .unwrap();
                }
            });
        }

        macro_rules! display_abil {
            ($display:expr, $target:expr, $($path:ident),+) => {
                let mut x = 700.0;
                let step = 80.0;
                let y = 980.0;

                $(
                    x += step;

                    ImageRendererBuilder::new($display)
                        .with_texture_bytes(include_bytes!(
                            concat!("../../assets/tracer/storm_ui_icon_tracer_", stringify!($path), ".png")
                        ))
                        .with_position([x, y])
                        .with_size([128.0 / 2.0, 128.0 / 2.0]) // 64x64
                        .build()
                        .unwrap()
                        .draw(&mut $target)
                        .unwrap();
                )+
            };
        }

        display_abil!(display, target, blink, melee, recall, pulsebomb, reload);


        target.finish().unwrap();
    }

    fn handle_window_event(
        &mut self,
        event: &glium::winit::event::WindowEvent,
        _window: &glium::winit::window::Window,
    ) {
        match event {
            glium::winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.mouse.position = (*position).into();
                let mut server_info = GAME.write().unwrap();

                server_info.mouse.position = (*position).into();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.handle_key_input(event);

                            let mut server_info = GAME.write().unwrap();

                            if !server_info.keyboard.held.contains(&code) {
                                server_info.keyboard.pressed.insert(code);
                            }
                            server_info.keyboard.held.insert(code);
                        }
                        ElementState::Released => {
                            self.update_shift_state(event);
                            let mut server_info = GAME.write().unwrap();

                            server_info.keyboard.held.remove(&code);
                            server_info.keyboard.released.insert(code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    let mut server_info = GAME.write().unwrap();

                    if !server_info.mouse.held.contains(button) {
                        server_info.mouse.pressed.insert(*button);
                    }
                    server_info.mouse.held.insert(*button);
                }
                ElementState::Released => {
                    let mut server_info = GAME.write().unwrap();

                    server_info.mouse.held.remove(button);
                    server_info.mouse.released.insert(*button);
                }
            },

            _ => (),
        }
    }
}
