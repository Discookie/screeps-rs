use std::error::Error;
use screeps::{
    prelude::*,
    objects::{
        Creep,
        StructureController
    },
    constants::*
};

use crate::traits::{
    Task,
    FlagProcessor
};

/// A creep moves to its room's controller, and upgrades it.
pub struct TaskUpgrade;

impl TaskUpgrade {
    pub fn new() -> TaskUpgrade {
        TaskUpgrade{}
    }
}

impl FlagProcessor for TaskUpgrade {}

impl Task for TaskUpgrade {
    fn run(&self, creep: &Creep) -> Result<bool, Box<dyn Error>> {
        let controller: StructureController = creep.room().controller().ok_or("there is no controller")?;

        if creep.upgrade_controller(&controller) == ReturnCode::NotInRange {
            creep.move_to(&controller);
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "upgrade"
    }
}