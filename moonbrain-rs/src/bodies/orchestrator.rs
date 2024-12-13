use godot::{obj::WithBaseField, prelude::*};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::gravity::{self, GravityBody};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Orchestrator {
    #[export]
    #[init(val = 9.8)]
    gravity_constant: f32,

    #[export]
    sun: Option<Gd<Node3D>>,

    base: Base<Node>,
}

impl Orchestrator {
    pub fn sun_pos(&self) -> Option<Vector3> {
        self.sun.as_ref().map(|x| x.get_global_position())
    }
}
#[godot_api]
impl INode for Orchestrator {
    fn ready(&mut self) {
        self.base_mut()
            .add_to_group_ex("Orchestrator")
            .persistent(true)
            .done();
    }

    fn process(&mut self, _dt: f64) {
        let mut nodes: Vec<_> = self
            .base()
            .get_tree()
            .unwrap()
            .get_nodes_in_group(gravity::GravityBody::GROUP)
            .iter_shared()
            .filter_map(|n| n.try_cast::<GravityBody>().ok())
            .collect();

        for i in 0..nodes.len() {
            let pos = nodes[i].get_global_position() + nodes[i].get_center_of_mass();
            let mass = nodes[i].get_mass();

            let force = nodes
                .iter()
                .filter_map(|other| {
                    let other_mass = other.get_mass();
                    let other_pos = other.get_global_position();

                    let dir = pos.try_direction_to(other_pos)?;
                    let dist2 = pos.distance_squared_to(other_pos);

                    Some(self.gravity_constant * dir * other_mass * mass / dist2)
                })
                .sum();

            nodes[i].apply_central_force(force);
        }
    }
}
