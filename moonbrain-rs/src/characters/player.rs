use std::f32;

use godot::builtin::math::FloatExt;
use godot::classes::input::MouseMode;
use godot::classes::{
    CharacterBody3D, ICharacterBody3D, InputEvent, InputEventMouseMotion, Marker3D,
};
use godot::obj::WithBaseField;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CharacterBody3D)]
pub struct Player {
    #[export]
    pivot: Option<Gd<Marker3D>>,

    #[export]
    camera: Option<Gd<Camera3D>>,

    #[export]
    look_sensitivity: f32,

    #[export]
    de_accel: f32,

    #[export]
    acceleration: f32,

    #[export]
    max_speed: f32,

    _base: Base<CharacterBody3D>,
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn ready(&mut self) {
        Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
    }

    fn process(&mut self, dt: f64) {
        let dt = dt as f32;

        let mut input = Input::singleton();

        if input.is_action_just_pressed("ui_select") {
            input.set_mouse_mode(MouseMode::CAPTURED);
        }

        if input.is_action_just_pressed("ui_cancel") {
            input.set_mouse_mode(MouseMode::VISIBLE);
        }

        self.base_mut().move_and_slide();
        let mut vel = self.base().get_velocity();

        let aim = self.base().get_global_basis().to_cols();
        let forward = aim[2];
        let up = Vector3::UP;
        let right = aim[0];

        let dir = Vector3::new(
            input.get_axis("forward", "back"),
            input.get_axis("left", "right"),
            input.get_axis("down", "up"),
        )
        .normalized_or_zero();

        if !dir.is_zero_approx() && vel.length() < self.max_speed {
            vel += dt * (dir.x * forward + dir.y * right + dir.z * up) * self.acceleration;
        } else {
            vel *= self.de_accel;
        }

        self.base_mut().set_velocity(vel);
    }

    fn input(&mut self, ev: Gd<InputEvent>) {
        if Input::singleton().get_mouse_mode() != MouseMode::CAPTURED {
            return;
        }

        let Ok(ev) = ev.try_cast::<InputEventMouseMotion>() else {
            return;
        };

        let vel = -ev.get_relative()
            * self.look_sensitivity
            * self.base().get_process_delta_time() as f32;

        let mut rot = self.base_mut().get_rotation();
        rot.y += vel.x;
        rot.y = rot.y.fposmod(f32::consts::TAU);
        self.base_mut().set_rotation(rot);

        let mut pivot = self.head_pivot();

        let mut rot = self.head_pivot().get_rotation();
        rot.x += vel.y;
        rot.x = rot.x.clamp(-f32::consts::FRAC_PI_2, f32::consts::FRAC_PI_2);
        pivot.set_rotation(rot);
    }
}

#[godot_api]
impl Player {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if the camera is not set in the editor
    #[must_use]
    pub fn camera(&self) -> Gd<Camera3D> {
        self.camera
            .clone()
            .expect("Camera is not correctly attached to player")
    }

    /// # Panics
    ///
    /// Panics if.the marker is not set in the editor
    #[must_use]
    pub fn head_pivot(&self) -> Gd<Marker3D> {
        self.pivot
            .clone()
            .expect("Pivot is not correctly attached to player")
    }
}
