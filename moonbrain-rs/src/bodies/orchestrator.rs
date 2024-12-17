use godot::{builtin::math::FloatExt, classes::Engine, obj::WithBaseField, prelude::*};
use rayon::iter::ParallelIterator;

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
    #[must_use]
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

    fn physics_process(&mut self, dt: f64) {
        let mut nodes: Vec<_> = self
            .base()
            .get_tree()
            .unwrap()
            .get_nodes_in_group(gravity::GravityBody::GROUP)
            .iter_shared()
            .filter_map(|n| n.try_cast::<GravityBody>().ok())
            .collect();

        let target = if Input::singleton().is_action_pressed("time_scale") {
            10.
        } else {
            1.
        };

        let scale = Engine::singleton().get_time_scale();
        Engine::singleton().set_time_scale(scale.lerp(target, dt / scale));

        let dt = dt as f32;
        for i in 0..nodes.len() {
            let pos = nodes[i].get_global_position();
            let mass = nodes[i].bind().get_mass();

            let mut linear_vel = nodes[i].get_constant_linear_velocity();

            for (j, other) in nodes.iter().enumerate() {
                if i == j {
                    continue;
                }

                let other_mass = other.bind().get_mass();
                let other_pos = other.get_global_position();

                let dir = pos.direction_to(other_pos);
                let distance = pos.distance_to(other_pos);

                let force = self.gravity_constant * mass * dir * other_mass / (distance * distance);
                linear_vel += force * dt / mass;
            }

            let node = &mut nodes[i];

            let mut node = node.bind_mut();
            node.base_mut().set_constant_linear_velocity(linear_vel);
        }
    }
}
