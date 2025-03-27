use chumsky::container::Container;
use color_eyre::owo_colors::OwoColorize;

use super::op::Instruction;

pub struct Chunk<Extra> {
    instructions: Vec<(Instruction, Option<Extra>)>,
}
