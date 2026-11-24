//! Format binaire PBC : chargement et validation d'un fichier .pbc.

mod fault;
mod instruction;
mod program;
mod reader;

pub use fault::Fault;
pub use instruction::{Instruction, decode};
pub use program::Program;
