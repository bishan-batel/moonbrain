use godot::{
    classes::{IStaticBody3D, PhysicsDirectBodyState3D, StaticBody3D},
    obj::WithBaseField,
    prelude::*,
};

use super::orchestrator::Orchestrator;

#[derive(GodotClass)]
#[class(init, base=StaticBody3D)]
#[allow(clippy::module_name_repetitions)]
pub struct GravityBody {
    #[export]
    mass: f32,

    #[export]
    orbit_point: Option<Gd<GravityBody>>,
    _base: Base<StaticBody3D>,
}

#[godot_api]
impl GravityBody {
    pub const GROUP: &str = "gravity_body";

    #[func]
    pub fn get_solar_system(&self) -> Option<Gd<Orchestrator>> {
        self.base()
            .get_tree()?
            .get_first_node_in_group("Orchestrator")?
            .try_cast::<Orchestrator>()
            .ok()
    }

    pub fn get_orbital_velocity_for(&self, source_body: &Gd<GravityBody>) -> Option<Vector3> {
        let solar_system = self.get_solar_system()?;

        // get the source body we are orbiting around
        let inherited_orbital_velocity = source_body
            .bind()
            .get_orbit_point()
            .as_ref()
            .and_then(|x| self.get_orbital_velocity_for(x));

        // orbit centre
        let source_pos = source_body.get_global_position();

        // our position
        let pos = self.base().get_global_position();

        // global gravitational constant
        let grav_constant = solar_system.bind().get_gravity_constant();

        // grav parameter for the body
        let grav_parameter = grav_constant * (source_body.bind().mass + self.mass);

        let distance = source_pos.distance_to(pos);
        let direction = source_pos.direction_to(pos);

        let vel_dir = direction.cross(Vector3::UP).normalized_or_zero();
        let strength = (grav_parameter / distance).sqrt();

        Some(vel_dir * strength + inherited_orbital_velocity.unwrap_or_default())
    }

    pub fn get_orbital_velocity(&self) -> Option<Vector3> {
        self.get_orbital_velocity_for(self.orbit_point.as_ref()?)
    }
}

#[godot_api]
impl IStaticBody3D for GravityBody {
    fn ready(&mut self) {
        self.base_mut()
            .add_to_group_ex(Self::GROUP)
            .persistent(true)
            .done();

        let orbital_velocity = self.get_orbital_velocity().unwrap_or_default();
        self.base_mut()
            .set_constant_linear_velocity(orbital_velocity);
    }

    fn physics_process(&mut self, dt: f64) {
        let sun = self
            .base()
            .get_tree()
            .unwrap()
            .get_first_node_in_group("Orchestrator")
            .unwrap()
            .try_cast::<Orchestrator>()
            .ok()
            .unwrap()
            .bind()
            .get_sun()
            .unwrap();

        self.base_mut().global_translate(-sun.get_global_position());

        let vel = self.base().get_constant_linear_velocity();
        self.base_mut().global_translate(vel * dt as f32);
    }
}
