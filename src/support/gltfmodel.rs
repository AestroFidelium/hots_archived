use anyhow::{Context, Result};
use cgmath::{
    EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Rotation3, SquareMatrix, Vector3,
    Zero,
};
use glium::index::PrimitiveType;
use glium::texture::{DepthTexture2d, RawImage2d, SrgbTexture2d};
use glium::{
    Display, DrawError, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer,
    implement_vertex, uniform,
};
use gltf::animation::util::ReadOutputs;
use gltf::{Glb, Gltf};
use glutin::surface::WindowSurface;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, normal, tex_coords);

#[derive(Clone, Debug)]
pub struct GltfNode {
    pub name: Option<String>,
    pub mesh_index: Option<usize>,
    pub translation: Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct GltfAnimationChannel {
    pub target_node: usize,
    pub path: String,
    pub input_times: Vec<f32>,
    pub output_values: Vec<[f32; 4]>,
}

#[derive(Clone, Debug)]
pub struct GltfAnimation {
    pub name: Option<String>,
    pub channels: Vec<GltfAnimationChannel>,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub struct GltfSkin {
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Matrix4<f32>>,
}

pub struct GltfPrimitive {
    pub vertices: VertexBuffer<Vertex>,
    pub indices: IndexBuffer<u32>,
    pub bind_positions: Vec<[f32; 3]>,
    pub bind_normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub joints: Option<Vec<[u16; 4]>>,
    pub weights: Option<Vec<[f32; 4]>>,
    pub skin_index: Option<usize>,
}

impl Clone for GltfPrimitive {
    fn clone(&self) -> Self {
        // Клонирование буферов требует пересоздания из исходных данных
        let vertices: Vec<Vertex> = self
            .bind_positions
            .iter()
            .zip(self.bind_normals.iter())
            .zip(self.tex_coords.iter())
            .map(|((&pos, &norm), &uv)| Vertex {
                position: pos,
                normal: norm,
                tex_coords: uv,
            })
            .collect();

        let display = self.vertices.get_context();
        let vertex_buffer =
            VertexBuffer::persistent(display, &vertices).expect("Failed to clone vertex buffer");

        let indices: Vec<u32> = self
            .indices
            .read()
            .expect("Failed to read indices")
            .to_vec();
        let index_buffer = IndexBuffer::persistent(display, PrimitiveType::TrianglesList, &indices)
            .expect("Failed to clone index buffer");

        Self {
            vertices: vertex_buffer,
            indices: index_buffer,
            bind_positions: self.bind_positions.clone(),
            bind_normals: self.bind_normals.clone(),
            tex_coords: self.tex_coords.clone(),
            joints: self.joints.clone(),
            weights: self.weights.clone(),
            skin_index: self.skin_index,
        }
    }
}

pub struct GltfModel {
    pub primitives: Vec<GltfPrimitive>,
    pub nodes: Vec<GltfNode>,
    pub animations: Vec<GltfAnimation>,
    pub skins: Vec<GltfSkin>,
    pub texture: Rc<SrgbTexture2d>,

    // Animation state (unified)
    current_time: f32,
    playing: bool,
    current_animation: usize,
    loop_animation: bool,

    // Cached matrices for performance
    cached_world_matrices: Vec<Matrix4<f32>>,
    cached_skin_matrices: Vec<Vec<Matrix4<f32>>>,
    matrices_dirty: bool,

    // Global transform parameters
    pub global_position: Vector3<f32>,
    pub global_rotation: Vector3<f32>,
    pub global_scale: Vector3<f32>,
    pub global_color: [f32; 4],
}

fn node_local_matrix(node: &GltfNode) -> Matrix4<f32> {
    let t = Matrix4::from_translation(node.translation);
    let r = Matrix4::from(node.rotation);
    let s = Matrix4::from_nonuniform_scale(node.scale.x, node.scale.y, node.scale.z);
    t * r * s
}

pub fn load_gltf_model_from_bytes(
    display: &Display<WindowSurface>,
    glb_bytes: &[u8],
    texture_bytes: &[u8],
    external_anim_bytes: Option<&[u8]>,
) -> Result<GltfModel> {
    let glb = Glb::from_slice(glb_bytes).context("Failed to parse GLB file")?;
    let gltf = Gltf::from_slice(&glb.json).context("Failed to parse GLTF JSON")?;
    let bin = glb.bin.context("No binary chunk in GLB")?;

    let dyn_img =
        image::load_from_memory(texture_bytes).context("Failed to decode texture bytes")?;
    let rgba = dyn_img.to_rgba8();
    let dims = rgba.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dims);
    let texture = SrgbTexture2d::new(display, raw).context("Failed to create SrgbTexture2d")?;

    let mut primitives: Vec<GltfPrimitive> = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| {
                if buffer.index() == 0 {
                    Some(&*bin)
                } else {
                    None
                }
            });

            let positions_iter = reader
                .read_positions()
                .context("No positions in primitive")?;
            let positions: Vec<[f32; 3]> = positions_iter.collect();
            let pos_count = positions.len();

            let normals: Vec<[f32; 3]> = if let Some(norm_iter) = reader.read_normals() {
                norm_iter.collect()
            } else {
                vec![[0.0, 1.0, 0.0]; pos_count]
            };

            let tex_coords: Vec<[f32; 2]> = if let Some(tc) = reader.read_tex_coords(0) {
                tc.into_f32().collect()
            } else {
                vec![[0.0, 0.0]; pos_count]
            };

            if positions.len() != normals.len() || positions.len() != tex_coords.len() {
                return Err(anyhow::anyhow!("Attribute length mismatch in primitive"));
            }

            let joints: Option<Vec<[u16; 4]>> = if let Some(j_iter) = reader.read_joints(0) {
                let arr: Vec<[u16; 4]> = j_iter.into_u16().collect();
                if arr.len() == pos_count {
                    Some(arr)
                } else {
                    None
                }
            } else {
                None
            };

            let weights: Option<Vec<[f32; 4]>> = if let Some(w_iter) = reader.read_weights(0) {
                let arr: Vec<[f32; 4]> = w_iter.into_f32().collect();
                if arr.len() == pos_count {
                    Some(arr)
                } else {
                    None
                }
            } else {
                None
            };

            let mut vertices = Vec::with_capacity(pos_count);
            for i in 0..pos_count {
                vertices.push(Vertex {
                    position: positions[i],
                    normal: normals[i],
                    tex_coords: tex_coords[i],
                });
            }

            let indices: Vec<u32> = if let Some(indices_iter) = reader.read_indices() {
                indices_iter.into_u32().collect()
            } else {
                (0..vertices.len() as u32).collect()
            };

            let vertex_buffer = VertexBuffer::persistent(display, &vertices)
                .context("Failed to create vertex buffer")?;
            let index_buffer =
                IndexBuffer::persistent(display, PrimitiveType::TrianglesList, &indices)
                    .context("Failed to create index buffer")?;

            primitives.push(GltfPrimitive {
                vertices: vertex_buffer,
                indices: index_buffer,
                bind_positions: positions,
                bind_normals: normals,
                tex_coords,
                joints,
                weights,
                skin_index: None,
            });
        }
    }

    let mut nodes = Vec::new();
    for node in gltf.nodes() {
        let transform = node.transform().decomposed();
        nodes.push(GltfNode {
            name: node.name().map(|s| s.to_string()),
            mesh_index: node.mesh().map(|m| m.index()),
            translation: Vector3::from(transform.0),
            rotation: cgmath::Quaternion::from(transform.1),
            scale: Vector3::from(transform.2),
            children: node.children().map(|c| c.index()).collect(),
        });
    }

    let mut skins: Vec<GltfSkin> = Vec::new();
    for skin in gltf.skins() {
        let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
        let reader = skin.reader(|buffer| {
            if buffer.index() == 0 {
                Some(&*bin)
            } else {
                None
            }
        });
        let inverse_bind_matrices: Vec<Matrix4<f32>> =
            if let Some(iter) = reader.read_inverse_bind_matrices() {
                iter.map(Matrix4::from).collect()
            } else {
                vec![Matrix4::identity(); joints.len()]
            };
        skins.push(GltfSkin {
            joints,
            inverse_bind_matrices,
        });
    }

    let mut mesh_to_primitive_indices: HashMap<usize, Vec<usize>> = HashMap::new();
    {
        let mut cursor = 0usize;
        for mesh in gltf.meshes() {
            let m_idx = mesh.index();
            let prim_count = mesh.primitives().count();
            let mut vec_idx = Vec::new();
            for _ in 0..prim_count {
                vec_idx.push(cursor);
                cursor += 1;
            }
            mesh_to_primitive_indices.insert(m_idx, vec_idx);
        }
    }

    for node in gltf.nodes() {
        if let (Some(mesh), Some(skin_ref)) = (node.mesh(), node.skin()) {
            let mesh_idx = mesh.index();
            let skin_idx = skin_ref.index();
            if let Some(prim_idxs) = mesh_to_primitive_indices.get(&mesh_idx) {
                for &prim_glob in prim_idxs {
                    if prim_glob < primitives.len() {
                        primitives[prim_glob].skin_index = Some(skin_idx);
                    }
                }
            }
        }
    }

    let mut animations: Vec<GltfAnimation> = Vec::new();
    for anim in gltf.animations() {
        let mut channels: Vec<GltfAnimationChannel> = Vec::new();
        let mut duration: f32 = 0.0;
        for channel in anim.channels() {
            let target = channel.target();
            let reader = channel.reader(|buffer| {
                if buffer.index() == 0 {
                    Some(&*bin)
                } else {
                    None
                }
            });

            let input_times: Vec<f32> = match reader.read_inputs() {
                Some(it) => it.collect(),
                None => continue,
            };
            if let Some(&last) = input_times.last() {
                duration = duration.max(last);
            }

            let path = match target.property() {
                gltf::animation::Property::Translation => "translation",
                gltf::animation::Property::Rotation => "rotation",
                gltf::animation::Property::Scale => "scale",
                gltf::animation::Property::MorphTargetWeights => "weights",
            }
            .to_string();

            let output_values: Vec<[f32; 4]> = match reader.read_outputs().unwrap() {
                ReadOutputs::Translations(v) => v.map(|t| [t[0], t[1], t[2], 0.0]).collect(),
                ReadOutputs::Rotations(v) => {
                    v.into_f32().map(|r| [r[0], r[1], r[2], r[3]]).collect()
                }
                ReadOutputs::Scales(v) => v.map(|s| [s[0], s[1], s[2], 0.0]).collect(),
                _ => continue,
            };

            channels.push(GltfAnimationChannel {
                target_node: target.node().index(),
                path,
                input_times,
                output_values,
            });
        }

        animations.push(GltfAnimation {
            name: anim.name().map(|s| s.to_string()),
            channels,
            duration,
        });
    }

    if primitives.is_empty() {
        return Err(anyhow::anyhow!("No primitives found in GLTF model"));
    }

    let node_count = nodes.len();
    let mut model = GltfModel {
        primitives,
        nodes,
        animations,
        skins,
        texture: Rc::new(texture),
        current_time: 0.0,
        playing: false,
        current_animation: 0,
        loop_animation: true,
        cached_world_matrices: vec![Matrix4::identity(); node_count],
        cached_skin_matrices: Vec::new(),
        matrices_dirty: true,
        global_position: Vector3::zero(),
        global_rotation: Vector3::zero(),
        global_scale: Vector3::new(1.0, 1.0, 1.0),
        global_color: [1.0, 1.0, 1.0, 1.0],
    };

    if let Some(anim_bytes) = external_anim_bytes {
        model.merge_animations_from_bytes(anim_bytes)?;
    }

    Ok(model)
}

pub struct GltfModelBuilder<'a> {
    pub model: &'a GltfModel,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub color: [f32; 4],
}

impl<'a> GltfModelBuilder<'a> {
    pub fn new(model: &'a GltfModel) -> Self {
        Self {
            model,
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn position(mut self, pos: Vector3<f32>) -> Self {
        self.position = pos;
        self
    }

    pub fn rotation(mut self, rot: Vector3<f32>) -> Self {
        self.rotation = rot;
        self
    }

    pub fn scale(mut self, scl: Vector3<f32>) -> Self {
        self.scale = scl;
        self
    }

    pub fn color(mut self, col: [f32; 4]) -> Self {
        self.color = col;
        self
    }

    pub fn draw_animated<S: Surface>(
        &self,
        display: &Display<WindowSurface>,
        target: &mut S,
        program: &Program,
        projection: Matrix4<f32>,
        view: Matrix4<f32>,
        light_pos: Vector3<f32>,
        view_pos: Vector3<f32>,
        shadow_map: &DepthTexture2d,
    ) -> Result<(), DrawError> {
        let draw_params = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let light_projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
        let light_view = Matrix4::look_at_rh(
            Point3::from_vec(light_pos),
            Point3::from_vec(Vector3::zero()),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let bias_matrix = Matrix4::from_nonuniform_scale(0.5, 0.5, 0.5)
            * Matrix4::from_translation(Vector3::new(0.5, 0.5, 0.5));

        let world_matrices = &self.model.cached_world_matrices;
        let skin_matrices_all = &self.model.cached_skin_matrices;

        for (node_idx, node) in self.model.nodes.iter().enumerate() {
            let mesh_index = match node.mesh_index {
                Some(i) => i,
                None => continue,
            };

            let builder_global_matrix = Matrix4::from_translation(self.position)
                * Matrix4::from(cgmath::Quaternion::from_angle_x(Rad(self.rotation.x)))
                * Matrix4::from(cgmath::Quaternion::from_angle_y(Rad(self.rotation.y)))
                * Matrix4::from(cgmath::Quaternion::from_angle_z(Rad(self.rotation.z)))
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

            let model_matrix = builder_global_matrix * world_matrices[node_idx];

            let depth_mvp = light_projection * light_view * model_matrix;
            let depth_bias_mvp = bias_matrix * depth_mvp;
            let mvp = projection * view * model_matrix;

            if mesh_index >= self.model.primitives.len() {
                continue;
            }
            let primitive = &self.model.primitives[mesh_index];

            let uniforms = uniform! {
                mvp: Into::<[[f32;4];4]>::into(mvp),
                depth_bias_mvp: Into::<[[f32;4];4]>::into(depth_bias_mvp),
                model_matrix: Into::<[[f32;4];4]>::into(model_matrix),
                model_color: self.color,
                light_loc: [light_pos.x, light_pos.y, light_pos.z],
                view_pos: [view_pos.x, view_pos.y, view_pos.z],
                shadow_map: shadow_map.sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
                shadow_map_size: shadow_map.get_width() as f32,
                diffuse_tex: self.model.texture.as_ref().sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Linear),
            };

            if let Some(skin_idx) = primitive.skin_index
                && skin_idx < skin_matrices_all.len()
            {
                let joint_matrices = &skin_matrices_all[skin_idx];
                let skinned_vertices = cpu_skin_vertices(
                    &primitive.bind_positions,
                    &primitive.bind_normals,
                    primitive.joints.as_ref(),
                    primitive.weights.as_ref(),
                    joint_matrices,
                    &primitive.tex_coords,
                );

                let skinned_vb = VertexBuffer::dynamic(display, &skinned_vertices)
                    .inspect_err(|&e| {
                        log::error!("Failed to create dynamic vertex buffer: {e}");
                    })
                    .unwrap();

                target.draw(
                    &skinned_vb,
                    &primitive.indices,
                    program,
                    &uniforms,
                    &draw_params,
                )?;
                continue;
            }

            target.draw(
                &primitive.vertices,
                &primitive.indices,
                program,
                &uniforms,
                &draw_params,
            )?;
        }

        Ok(())
    }
}

fn cpu_skin_vertices(
    bind_positions: &[[f32; 3]],
    bind_normals: &[[f32; 3]],
    joints_opt: Option<&Vec<[u16; 4]>>,
    weights_opt: Option<&Vec<[f32; 4]>>,
    joint_matrices: &[Matrix4<f32>],
    tex_coords: &[[f32; 2]],
) -> Vec<Vertex> {
    let n = bind_positions.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = bind_positions[i];
        let nrm = bind_normals[i];
        let uv = tex_coords.get(i).cloned().unwrap_or([0.0, 0.0]);

        if joints_opt.is_none() || weights_opt.is_none() {
            out.push(Vertex {
                position: p,
                normal: nrm,
                tex_coords: uv,
            });
            continue;
        }

        let joints = joints_opt.unwrap();
        let weights = weights_opt.unwrap();
        let joint_inds = joints.get(i).cloned().unwrap_or([0, 0, 0, 0]);
        let weight_vals = weights.get(i).cloned().unwrap_or([0.0, 0.0, 0.0, 0.0]);

        let mut pos_acc = cgmath::Vector4::new(0.0f32, 0.0, 0.0, 0.0);
        let mut n_acc = cgmath::Vector3::new(0.0f32, 0.0, 0.0);

        let pos4 = cgmath::Vector4::new(p[0], p[1], p[2], 1.0);
        let normal3 = cgmath::Vector3::new(nrm[0], nrm[1], nrm[2]);

        for k in 0..4 {
            let w = weight_vals[k];
            if w <= 0.0 {
                continue;
            }
            let j = joint_inds[k] as usize;
            if j >= joint_matrices.len() {
                continue;
            }
            let mat = joint_matrices[j];

            let tp = mat * pos4;
            pos_acc += tp * w;

            let mat3: Matrix3<f32> =
                Matrix3::from_cols(mat.x.truncate(), mat.y.truncate(), mat.z.truncate());
            let tn = mat3 * normal3;
            n_acc += tn * w;
        }

        let n_final = if n_acc.magnitude2() > 0.0 {
            let nn = n_acc.normalize();
            [nn.x, nn.y, nn.z]
        } else {
            nrm
        };

        let p_final = [pos_acc.x, pos_acc.y, pos_acc.z];

        out.push(Vertex {
            position: p_final,
            normal: n_final,
            tex_coords: uv,
        });
    }
    out
}

impl GltfModel {
    // ========== Transform Setters (Optimized) ==========

    /// Set global position
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.global_position = position;
    }

    /// Set position by components
    pub fn set_position_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.global_position = Vector3::new(x, y, z);
    }

    /// Translate position by offset
    pub fn translate(&mut self, offset: Vector3<f32>) {
        self.global_position += offset;
    }

    /// Set global rotation (in radians)
    pub fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.global_rotation = rotation;
    }

    /// Set rotation by components (in radians)
    pub fn set_rotation_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.global_rotation = Vector3::new(x, y, z);
    }

    /// Rotate by offset (in radians)
    pub fn rotate(&mut self, offset: Vector3<f32>) {
        self.global_rotation += offset;
    }

    /// Set rotation in degrees
    pub fn set_rotation_degrees(&mut self, degrees: Vector3<f32>) {
        self.global_rotation = Vector3::new(
            degrees.x.to_radians(),
            degrees.y.to_radians(),
            degrees.z.to_radians(),
        );
    }

    /// Set global scale
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.global_scale = scale;
    }

    /// Set uniform scale
    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.global_scale = Vector3::new(scale, scale, scale);
    }

    /// Set scale by components
    pub fn set_scale_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.global_scale = Vector3::new(x, y, z);
    }

    /// Multiply scale by factor
    pub fn scale_by(&mut self, factor: Vector3<f32>) {
        self.global_scale.x *= factor.x;
        self.global_scale.y *= factor.y;
        self.global_scale.z *= factor.z;
    }

    /// Set global color (RGBA)
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.global_color = color;
    }

    /// Set color from RGB (alpha = 1.0)
    pub fn set_color_rgb(&mut self, r: f32, g: f32, b: f32) {
        self.global_color = [r, g, b, 1.0];
    }

    /// Set color from RGBA components
    pub fn set_color_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.global_color = [r, g, b, a];
    }

    /// Set alpha (transparency)
    pub fn set_alpha(&mut self, alpha: f32) {
        self.global_color[3] = alpha;
    }

    /// Get current position
    pub fn position(&self) -> Vector3<f32> {
        self.global_position
    }

    /// Get current rotation (in radians)
    pub fn rotation(&self) -> Vector3<f32> {
        self.global_rotation
    }

    /// Get current scale
    pub fn scale(&self) -> Vector3<f32> {
        self.global_scale
    }

    /// Get current color
    pub fn color(&self) -> [f32; 4] {
        self.global_color
    }

    // ========== Animation Control ==========

    /// Play animation by index
    pub fn play_animation(&mut self, animation_index: usize) {
        if animation_index < self.animations.len() {
            self.current_animation = animation_index;
            self.current_time = 0.0;
            self.playing = true;
            self.matrices_dirty = true;
        }
    }

    /// Play animation by name
    pub fn play_animation_by_name(&mut self, name: &str) {
        if let Some(idx) = self
            .animations
            .iter()
            .position(|a| a.name.as_deref() == Some(name))
        {
            self.play_animation(idx);
        }
    }

    /// Pause animation
    pub fn pause_animation(&mut self) {
        self.playing = false;
    }

    /// Resume animation
    pub fn resume_animation(&mut self) {
        self.playing = true;
    }

    /// Stop animation and reset to start
    pub fn stop_animation(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
        self.matrices_dirty = true;
    }

    /// Set looping behavior
    pub fn set_loop(&mut self, should_loop: bool) {
        self.loop_animation = should_loop;
    }

    /// Get current animation index
    pub fn current_animation(&self) -> usize {
        self.current_animation
    }

    /// Get current animation time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Set animation time manually
    pub fn set_time(&mut self, time: f32) {
        if self.current_animation < self.animations.len() {
            let duration = self.animations[self.current_animation].duration;
            self.current_time = if self.loop_animation && duration > 0.0 {
                time % duration
            } else {
                time.min(duration)
            };
            self.matrices_dirty = true;
        }
    }

    /// Update animation (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        if !self.playing || self.current_animation >= self.animations.len() {
            return;
        }

        let anim = &self.animations[self.current_animation];
        if anim.duration <= 0.0 {
            return;
        }

        self.current_time += delta_time;

        if self.current_time >= anim.duration {
            if self.loop_animation {
                self.current_time %= anim.duration;
            } else {
                self.current_time = anim.duration;
                self.playing = false;
            }
        }

        // Apply animation to nodes
        for channel in &anim.channels {
            if channel.target_node >= self.nodes.len() {
                continue;
            }

            let (_t1, _t2, idx, alpha) = match self.find_animation_keyframes(channel) {
                Some(data) => data,
                None => continue,
            };

            let v1 = channel.output_values[idx];
            let v2 = channel.output_values[idx + 1];

            let node = &mut self.nodes[channel.target_node];

            match channel.path.as_str() {
                "translation" => {
                    node.translation = Vector3::new(
                        v1[0] + (v2[0] - v1[0]) * alpha,
                        v1[1] + (v2[1] - v1[1]) * alpha,
                        v1[2] + (v2[2] - v1[2]) * alpha,
                    );
                }
                "rotation" => {
                    let q1 = cgmath::Quaternion::new(v1[3], v1[0], v1[1], v1[2]);
                    let q2 = cgmath::Quaternion::new(v2[3], v2[0], v2[1], v2[2]);
                    node.rotation = q1.slerp(q2, alpha);
                }
                "scale" => {
                    node.scale = Vector3::new(
                        v1[0] + (v2[0] - v1[0]) * alpha,
                        v1[1] + (v2[1] - v1[1]) * alpha,
                        v1[2] + (v2[2] - v1[2]) * alpha,
                    );
                }
                _ => {}
            }
        }

        self.matrices_dirty = true;
    }

    fn find_animation_keyframes(
        &self,
        channel: &GltfAnimationChannel,
    ) -> Option<(f32, f32, usize, f32)> {
        let window = channel
            .input_times
            .windows(2)
            .enumerate()
            .find(|(_, w)| self.current_time >= w[0] && self.current_time <= w[1]);

        if let Some((idx, w)) = window {
            let t1 = w[0];
            let t2 = w[1];
            let alpha = if (t2 - t1).abs() < std::f32::EPSILON {
                0.0
            } else {
                (self.current_time - t1) / (t2 - t1)
            };
            Some((t1, t2, idx, alpha))
        } else {
            None
        }
    }

    // ========== Matrix Computation ==========

    /// Update cached matrices (called automatically when needed)
    fn update_matrices(&mut self) {
        if !self.matrices_dirty {
            return;
        }

        self.cached_world_matrices = self.compute_node_world_matrices();
        self.cached_skin_matrices =
            self.compute_skin_matrices_from_world(&self.cached_world_matrices);
        self.matrices_dirty = false;
    }

    fn compute_node_world_matrices(&self) -> Vec<Matrix4<f32>> {
        let n = self.nodes.len();
        let mut world: Vec<Option<Matrix4<f32>>> = vec![None; n];

        let mut parent: Vec<Option<usize>> = vec![None; n];
        for (i, node) in self.nodes.iter().enumerate() {
            for &child in &node.children {
                parent[child] = Some(i);
            }
        }

        fn calc(
            i: usize,
            model: &GltfModel,
            parent: &Vec<Option<usize>>,
            world: &mut Vec<Option<Matrix4<f32>>>,
        ) -> Matrix4<f32> {
            if let Some(m) = &world[i] {
                return *m;
            }
            let local = node_local_matrix(&model.nodes[i]);
            let mat = if let Some(p) = parent[i] {
                calc(p, model, parent, world) * local
            } else {
                local
            };
            world[i] = Some(mat);
            mat
        }

        for i in 0..n {
            if world[i].is_none() {
                let _ = calc(i, self, &parent, &mut world);
            }
        }

        world
            .into_iter()
            .map(|o| o.unwrap_or_else(Matrix4::identity))
            .collect()
    }

    fn compute_skin_matrices_from_world(
        &self,
        world_matrices: &Vec<Matrix4<f32>>,
    ) -> Vec<Vec<Matrix4<f32>>> {
        let mut all: Vec<Vec<Matrix4<f32>>> = Vec::with_capacity(self.skins.len());
        for skin in &self.skins {
            let mut mats: Vec<Matrix4<f32>> = Vec::with_capacity(skin.joints.len());
            for (i, &joint_node) in skin.joints.iter().enumerate() {
                let world = if joint_node < world_matrices.len() {
                    world_matrices[joint_node]
                } else {
                    Matrix4::identity()
                };
                let inv_bind = skin
                    .inverse_bind_matrices
                    .get(i)
                    .cloned()
                    .unwrap_or(Matrix4::identity());
                mats.push(world * inv_bind);
            }
            all.push(mats);
        }
        all
    }

    // ========== Drawing ==========

    /// Draw with current animation state (uses cached matrices)
    pub fn draw<S: Surface>(
        &mut self,
        display: &Display<WindowSurface>,
        target: &mut S,
        program: &Program,
        projection: Matrix4<f32>,
        view: Matrix4<f32>,
        light_pos: Vector3<f32>,
        view_pos: Vector3<f32>,
        shadow_map: &DepthTexture2d,
    ) -> Result<(), DrawError> {
        self.update_matrices();
        GltfModelBuilder::new(self)
            .position(self.global_position)
            .rotation(self.global_rotation)
            .scale(self.global_scale)
            .color(self.global_color)
            .draw_animated(
                display, target, program, projection, view, light_pos, view_pos, shadow_map,
            )
    }

    /// Draw with custom transform
    pub fn draw_with_transform<S: Surface>(
        &mut self,
        display: &Display<WindowSurface>,
        target: &mut S,
        program: &Program,
        projection: Matrix4<f32>,
        view: Matrix4<f32>,
        light_pos: Vector3<f32>,
        view_pos: Vector3<f32>,
        shadow_map: &DepthTexture2d,
        position: Vector3<f32>,
        rotation: Vector3<f32>,
        scale: Vector3<f32>,
        color: [f32; 4],
    ) -> Result<(), DrawError> {
        self.update_matrices();
        GltfModelBuilder::new(self)
            .position(position)
            .rotation(rotation)
            .scale(scale)
            .color(color)
            .draw_animated(
                display, target, program, projection, view, light_pos, view_pos, shadow_map,
            )
    }

    /// Draw with node animation (standard method - uses global transform parameters)
    pub fn draw_with_node_animation<S: Surface>(
        &mut self,
        display: &Display<WindowSurface>,
        target: &mut S,
        program: &Program,
        projection: Matrix4<f32>,
        view: Matrix4<f32>,
        light_pos: Vector3<f32>,
        view_pos: Vector3<f32>,
        shadow_map: &DepthTexture2d,
        _builder_global_matrix: Matrix4<f32>,
        _model_color: [f32; 4],
        _shadow_map_size: f32,
    ) -> Result<(), DrawError> {
        // Игнорируем переданные параметры и используем глобальные из модели
        self.draw(
            display, target, program, projection, view, light_pos, view_pos, shadow_map,
        )
    }

    // ========== Animation Merging ==========

    /// Merge animations from external GLB (name matching fallback index matching)
    pub fn merge_animations_from_bytes(&mut self, anim_bytes: &[u8]) -> Result<()> {
        let anim_glb = Glb::from_slice(anim_bytes).context("Failed to parse animation GLB")?;
        let anim_gltf =
            Gltf::from_slice(&anim_glb.json).context("Failed to parse animation GLTF")?;
        let bin = anim_glb
            .bin
            .as_ref()
            .context("No binary chunk in animation GLB")?;

        let mut name_to_index: HashMap<String, usize> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(name) = &node.name {
                name_to_index.insert(name.clone(), i);
            }
        }

        let mut anim_node_names: Vec<Option<String>> = Vec::new();
        for node in anim_gltf.nodes() {
            anim_node_names.push(node.name().map(|s| s.to_string()));
        }
        let anim_node_count = anim_node_names.len();
        let model_node_count = self.nodes.len();

        for anim in anim_gltf.animations() {
            let mut channels: Vec<GltfAnimationChannel> = Vec::new();
            let mut duration: f32 = 0.0;

            for channel in anim.channels() {
                let target = channel.target();
                let reader = channel.reader(|_buffer| Some(bin));

                let input_times: Vec<f32> = match reader.read_inputs() {
                    Some(it) => it.collect(),
                    None => continue,
                };
                if let Some(&last) = input_times.last() {
                    duration = duration.max(last);
                }

                let output_values: Vec<[f32; 4]> = match reader.read_outputs() {
                    Some(ReadOutputs::Translations(iter)) => {
                        iter.map(|t| [t[0], t[1], t[2], 0.0]).collect()
                    }
                    Some(ReadOutputs::Rotations(iter)) => {
                        iter.into_f32().map(|r| [r[0], r[1], r[2], r[3]]).collect()
                    }
                    Some(ReadOutputs::Scales(iter)) => {
                        iter.map(|s| [s[0], s[1], s[2], 0.0]).collect()
                    }
                    _ => continue,
                };

                let path = match target.property() {
                    gltf::animation::Property::Translation => "translation",
                    gltf::animation::Property::Rotation => "rotation",
                    gltf::animation::Property::Scale => "scale",
                    gltf::animation::Property::MorphTargetWeights => "weights",
                }
                .to_string();

                let anim_node_index = target.node().index();
                let model_node_index_opt: Option<usize> = if let Some(name) =
                    anim_node_names.get(anim_node_index).and_then(|n| n.clone())
                {
                    name_to_index.get(&name).copied()
                } else if anim_node_count == model_node_count && anim_node_index < model_node_count
                {
                    Some(anim_node_index)
                } else {
                    None
                };

                if let Some(model_node_index) = model_node_index_opt {
                    channels.push(GltfAnimationChannel {
                        target_node: model_node_index,
                        path,
                        input_times,
                        output_values,
                    });
                } else {
                    continue;
                }
            }

            if !channels.is_empty() {
                self.animations.push(GltfAnimation {
                    name: anim.name().map(|s| s.to_string()),
                    channels,
                    duration: if duration > 0.0 { duration } else { 0.0 },
                });
            }
        }

        Ok(())
    }

    // ========== Utility Methods ==========

    /// Clone the model (creates a complete copy with new GPU buffers)
    pub fn clone_model(&self) -> Self {
        let primitives = self.primitives.to_vec();

        Self {
            primitives,
            nodes: self.nodes.clone(),
            animations: self.animations.clone(),
            skins: self.skins.clone(),
            texture: Rc::clone(&self.texture),
            current_time: self.current_time,
            playing: self.playing,
            current_animation: self.current_animation,
            loop_animation: self.loop_animation,
            cached_world_matrices: self.cached_world_matrices.clone(),
            cached_skin_matrices: self.cached_skin_matrices.clone(),
            matrices_dirty: self.matrices_dirty,
            global_position: self.global_position,
            global_rotation: self.global_rotation,
            global_scale: self.global_scale,
            global_color: self.global_color,
        }
    }

    /// Get animation count
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }

    /// Get animation duration by index
    pub fn animation_duration(&self, index: usize) -> Option<f32> {
        self.animations.get(index).map(|a| a.duration)
    }

    /// Get animation name by index
    pub fn animation_name(&self, index: usize) -> Option<&str> {
        self.animations.get(index)?.name.as_deref()
    }

    /// Find animation index by name
    pub fn find_animation_index(&self, name: &str) -> Option<usize> {
        self.animations
            .iter()
            .position(|a| a.name.as_deref() == Some(name))
    }

    /// Force matrices recalculation on next draw
    pub fn invalidate_matrices(&mut self) {
        self.matrices_dirty = true;
    }
}

unsafe impl Sync for GltfModel {}
unsafe impl Send for GltfModel {}
