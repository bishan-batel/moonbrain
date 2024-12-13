use std::str::FromStr;

use godot::{
    classes::{
        EditorNode3DGizmo, Engine, IRigidBody3D, PhysicsBody3D, PhysicsDirectBodyState3D,
        RigidBody3D,
    },
    obj::{NewGd, WithBaseField},
    prelude::*,
};
use lazy_static::lazy_static;

use crate::is_tool;

use super::orchestrator::Orchestrator;

#[derive(GodotClass)]
#[class(init, base=RigidBody3D)]
#[allow(clippy::module_name_repetitions)]
pub struct GravityBody {
    #[export]
    initial_vel: Vector3,
    _base: Base<RigidBody3D>,
}

#[godot_api]
impl GravityBody {
    pub const GROUP: &str = "gravity_body";
}

#[godot_api]
impl IRigidBody3D for GravityBody {
    fn ready(&mut self) {
        let vel = self.initial_vel;
        self.base_mut().set_linear_velocity(vel);
        self.base_mut()
            .add_to_group_ex(Self::GROUP)
            .persistent(true)
            .done();
    }

    fn integrate_forces(&mut self, _state: Option<Gd<PhysicsDirectBodyState3D>>) {
        let Some(sun_pos) = (|| {
            self.base()
                .get_tree()?
                .get_first_node_in_group("Orchestrator")?
                .try_cast::<Orchestrator>()
                .ok()?
                .bind()
                .sun_pos()
        })() else {
            return;
        };

        self.base_mut().global_translate(-sun_pos);
    }
}
