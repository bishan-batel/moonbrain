use godot::{
    classes::{IMeshInstance3D, MeshInstance3D},
    prelude::*,
};

#[derive(Debug, GodotClass)]
#[class(init, tool, base = MeshInstance3D)]
pub struct Chunk {
    _base: Base<MeshInstance3D>,
}

#[godot_api]
impl IMeshInstance3D for Chunk {
    fn ready(&mut self) {
        let this = self.base().clone();

        self.base_mut().connect(
            "trigger_rebuild",
            &Callable::from_object_method(&this, "trigger_rebuild"),
        );
    }
}

#[godot_api]
impl Chunk {
    #[signal]
    pub fn trigger_rebuild(&mut self);

    pub fn rebuild(&mut self) {}
}
