use std::{io::Write, mem};

use bytebuffer::ByteBuffer;
use godot::{
    builtin::math::{assert_eq_approx, ApproxEq},
    classes::{
        rendering_device::{StorageBufferUsage, UniformType},
        ArrayMesh, IMeshInstance3D, ImmediateMesh, Mesh, MeshInstance2D, MeshInstance3D,
        RdShaderFile, RdUniform, RenderingDevice, RenderingServer,
    },
    global::wrap,
    meta::AsObjectArg,
    obj::{Base, NewGd, WithBaseField},
    prelude::*,
};
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};

use super::sphere::SphereGenerator;

#[derive(GodotClass, Debug)]
#[class(init, tool, base = Node3D)]
pub struct CelestialMesh {
    #[export]
    #[var(set = update_mesh, get = _get_update_mesh)]
    _update_mesh: bool,

    #[export]
    detail: u32,

    #[export]
    seed: f32,

    #[export]
    mesh_instance: Option<Gd<MeshInstance3D>>,

    #[export]
    compute_shader: Option<Gd<RdShaderFile>>,

    _base: Base<Node3D>,
}

#[godot_api]
impl CelestialMesh {
    #[must_use]
    pub fn mesh_instance(&self) -> Gd<MeshInstance3D> {
        self.mesh_instance
            .clone()
            .expect("Mesh instance is not set properly")
    }

    pub fn set_mesh(&mut self, mesh: impl AsObjectArg<Mesh>) {
        self.mesh_instance().set_mesh(mesh);
    }

    #[func]
    pub fn update_mesh(&mut self, rebuild: bool) {
        if !rebuild {
            return;
        }

        self.base_mut().update_configuration_warnings();

        let mut mesh = SphereGenerator::gen(self.detail as usize);

        {
            let mut rd = RenderingServer::singleton()
                .create_local_rendering_device()
                .expect("Failed to create local render device");

            let shader = {
                let bytecode = self
                    .compute_shader
                    .as_ref()
                    .expect("No compute shader set")
                    .get_spirv()
                    .expect("No Bytecode");
                rd.shader_create_from_spirv(&bytecode)
            };

            let vertices = &mut mesh.verticies;
            let length = vertices.len() as u32;

            let dispatch_groups = ((length as f32 / 16.).ceil() as u32).max(1);

            let vertex_buffer = {
                let mut buffer = PackedFloat32Array::new();

                for Vector3 { x, y, z } in vertices.iter() {
                    buffer.push(*x);
                    buffer.push(*y);
                    buffer.push(*z);
                }

                let buffer = buffer.to_byte_array();
                let amt = buffer.len();
                rd.storage_buffer_create_ex(amt as u32).data(&buffer).done()
            };
            assert!(vertex_buffer.is_valid(), "Failed to create vertex buffer");

            let vertices_uniform = {
                let mut uniform = RdUniform::new_gd();
                uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
                uniform.set_binding(0);
                uniform.add_id(vertex_buffer);
                uniform
            };

            let params_buffer = {
                let mut buffer = ByteBuffer::new();
                buffer.write_u32(length);
                buffer.write_f32(self.seed);

                rd.storage_buffer_create_ex(buffer.len() as u32)
                    .data(&PackedByteArray::from(buffer.as_bytes()))
                    .done()
            };

            let params_uniform = {
                let mut uniform = RdUniform::new_gd();
                uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
                uniform.set_binding(1);
                uniform.add_id(params_buffer);
                uniform
            };

            let uniform_set =
                rd.uniform_set_create(&array![&vertices_uniform, &params_uniform], shader, 0);
            assert!(uniform_set.is_valid(), "Invalid Uniform Set");

            let pipeline = rd.compute_pipeline_create(shader);
            let compute_list = rd.compute_list_begin();

            rd.compute_list_bind_compute_pipeline(compute_list, pipeline);
            rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0);
            // rd.compute_list_add_barrier(compute_list);

            // Setting the dispatch group numbers
            println!("Dispatching with {dispatch_groups}x1x1 Groups for {length} vertices");
            rd.compute_list_dispatch(compute_list, dispatch_groups, 1, 1);
            rd.compute_list_end();

            rd.submit();
            rd.sync();

            let output_bytes = rd.buffer_get_data(vertex_buffer).to_float32_array();

            assert_eq!(
                ByteBuffer::from_vec(rd.buffer_get_data(params_buffer).to_vec())
                    .read_u32()
                    .unwrap(),
                length
            );

            for (i, v) in vertices.iter_mut().enumerate() {
                v.x = output_bytes[i * 3];
                v.y = output_bytes[i * 3 + 1];
                v.z = output_bytes[i * 3 + 2];
            }

            rd.free_rid(vertex_buffer);
            rd.free_rid(params_buffer);
            rd.free_rid(pipeline);
            rd.free_rid(shader);
            rd.free();
        }

        self.set_mesh(&mesh.create_mesh());
    }

    #[func]
    #[must_use]
    fn _get_update_mesh(&self) -> bool {
        let _ = self;
        false
    }
}

#[godot_api]
impl INode3D for CelestialMesh {
    fn ready(&mut self) {}
}
