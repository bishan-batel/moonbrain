#![allow(clippy::cast_possible_truncation)]

use godot::prelude::*;

pub mod bodies;
pub mod characters;

struct MoonBrain;

#[gdextension]
unsafe impl ExtensionLibrary for MoonBrain {}

#[macro_export]
macro_rules! is_tool {
    () => {
        godot::classes::Engine::singleton().is_editor_hint()
    };
}
