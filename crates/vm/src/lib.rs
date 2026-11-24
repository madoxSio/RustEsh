//! Moteur d'exécution de la machine PBC.

mod machine;
mod stack;

pub use machine::{Machine, Step};
pub use stack::{STACK_SIZE, Stack};
