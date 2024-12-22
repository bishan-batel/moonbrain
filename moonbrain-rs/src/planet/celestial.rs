use std::mem;

use godot::{
    builtin::math::{assert_eq_approx, ApproxEq},
    classes::{
        rendering_device::{StorageBufferUsage, UniformType},
        ArrayMesh, IMeshInstance3D, ImmediateMesh, Mesh, MeshInstance2D, MeshInstance3D,
        RdShaderFile, RdUniform, RenderingDevice, RenderingServer,
    },
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
    pub fn update_mesh(&mut self, _: bool) {
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

            let total_invoke_groups = length;

            let buffer = {
                let mut buffer = PackedByteArray::new();

                // for b in bytemuck::bytes_of(&length) {
                //     buffer.push(*b);
                // }

                for v in vertices.iter() {
                    for b in bytemuck::bytes_of(&v.x) {
                        buffer.push(*b);
                    }
                    for b in bytemuck::bytes_of(&v.y) {
                        buffer.push(*b);
                    }
                    for b in bytemuck::bytes_of(&v.z) {
                        buffer.push(*b);
                    }
                }

                rd.storage_buffer_create_ex(buffer.len() as u32)
                    .data(&buffer)
                    .done()
            };

            let uniform = {
                let mut uniform = RdUniform::new_gd();
                uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
                uniform.set_binding(0);
                uniform.add_id(buffer);
                uniform
            };

            let uniform_set = rd.uniform_set_create(&array![&uniform], shader, 0);

            let pipeline = rd.compute_pipeline_create(shader);
            let compute_list = rd.compute_list_begin();

            rd.compute_list_bind_compute_pipeline(compute_list, pipeline);
            rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0);

            // Setting the dispatch group numbers
            {
                let groups = total_invoke_groups;

                rd.compute_list_dispatch(compute_list, groups, 1, 1);
            }
            rd.compute_list_end();

            rd.submit();
            rd.sync();

            let output_bytes = rd.buffer_get_data_ex(buffer).done().to_vec();

            // assert_eq!(bytemuck::from_bytes::<u32>(&output_bytes[..4]), &length);

            let vertices_bytes: &[f32] = bytemuck::cast_slice(&output_bytes[0..]);

            for i in 0..(length as usize) {
                let x: f32 = vertices_bytes[i * 3];
                let y: f32 = vertices_bytes[i * 3 + 1];
                let z: f32 = vertices_bytes[i * 3 + 2];
                vertices[i] = Vector3::new(x, y, z);
            }

            // *vertices = bytemuck::cast_slice::<_, f32>(vertices_bytes)
            //     .windows(3)
            //     .step_by(3)
            //     .map(|x| Vector3::new(x[0], x[1], x[2]))
            //     .collect();
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
