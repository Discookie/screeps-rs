use screeps::{
    objects::Creep,
    constants::Part
};
use std::error::Error;

/// Represents a creep's role.
/// Roles decide which tasks the creep should do in a given tick.
/// It also has a name, by which the creep can decide what role it has.
pub trait Role {
    fn name(&self) -> &'static str {
        "undefined"
    }

    fn limit(&self) -> i32;

    fn next_creep(&self) -> Vec<Part>;

    fn run_count(&self) -> i32;

    fn run(&self, creep: &Creep) -> Result<(), Box<Error>>;
}

/// Represents a creep's task.
/// A creep should only execute one task per tick.
pub trait Task {
    /// Returns true if the task was executed.
    fn run(&self, creep: &Creep) -> Result<bool, Box<Error>>;
}