#![allow(clippy::cast_possible_truncation)]

extern crate godot;
extern crate lazy_static;
extern crate rayon;
extern crate ron;
extern crate serde;

use godot::prelude::*;

pub mod bodies;
pub mod characters;
pub mod planet;

struct MoonBrain;

#[gdextension]
unsafe impl ExtensionLibrary for MoonBrain {}

#[macro_export]
macro_rules! is_tool {
    () => {
        godot::classes::Engine::singleton().is_editor_hint()
    };
}
