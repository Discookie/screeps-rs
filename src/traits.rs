use screeps::{
    memory::{
        MemoryReference,
        root
    },
    objects::{
        Creep,
        RoomPosition
    },
    constants::Part
};

use std::{
    error::Error,
    str::SplitWhitespace
};

pub trait FlagProcessor {
    fn flag(&self, _cmd: SplitWhitespace, _pos: RoomPosition) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }
}

/// Represents a creep's role.
/// Roles decide which tasks the creep should do in a given tick.
/// It also has a name, by which the creep can decide what role it has.
pub trait Role: FlagProcessor {
    fn name(&self) -> &'static str {
        "undefined"
    }

    fn limit(&self) -> i32;

    /// The part layout of the next creep to spawn
    fn next_creep(&self) -> Vec<Part>;

    fn run_count(&self) -> i32;

    fn run(&self, creep: &Creep) -> Result<(), Box<dyn Error>>;

    /// The lower this number, the more creeps there will be overall
    /// Default range: 10-100
    fn spawn_priority(&self) -> i32;
}

/// Represents a creep's task.
/// A creep should only execute one task per tick.
pub trait Task: FlagProcessor {
    /// Returns true if the task was executed.
    fn run(&self, creep: &Creep) -> Result<bool, Box<dyn Error>>;

    fn name(&self) -> &'static str {
        "undefined"
    }

    fn creep_memory(&self, creep: &Creep) -> Result<MemoryReference, Box<dyn Error>> {
        creep.memory()
             .dict_or_create("tasks")
             .and_then(|mem| mem.dict_or_create(self.name()))
          .or(Err(Box::from("error accessing creep memory")))
    }

    fn memory(&self) -> Result<MemoryReference, Box<dyn Error>> {
        root().dict_or_create("tasks")
             .and_then(|mem| mem.dict_or_create(self.name()))
          .or(Err(Box::from("error accessing global memory")))
    }
}
