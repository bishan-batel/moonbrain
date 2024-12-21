use godot::{
    classes::{ArrayMesh, IMeshInstance3D, ImmediateMesh, Mesh, MeshInstance2D, MeshInstance3D},
    meta::AsObjectArg,
    obj::{Base, NewGd, WithBaseField},
    prelude::*,
};

use super::generator::{self, PlanetGenerator};

#[derive(GodotClass, Debug)]
#[class(init, tool, base = Node3D)]
pub struct CelestialMesh {
    #[export]
    #[var(set = update_mesh, get = _get_update_mesh)]
    _update_mesh: bool,

    #[export]
    generator: Option<Gd<PlanetGenerator>>,

    #[export]
    mesh_instance: Option<Gd<MeshInstance3D>>,

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

        let mesh = self
            .generator
            .clone()
            .expect("No Generator Set")
            .bind()
            .generate();
        self.set_mesh(&mesh);
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
