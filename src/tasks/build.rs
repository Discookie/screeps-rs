use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep,
    constants::*
};

use crate::traits::Task;

/// A creep moves to the closest construction site, and attempts to build it.
pub struct TaskBuild;

impl TaskBuild {
    pub fn new() -> TaskBuild {
        TaskBuild{}
    }
}

impl Task for TaskBuild {
    fn run(&self, creep: &Creep) -> Result<bool, Box<Error>> {
        let sites = creep.room().find(find::CONSTRUCTION_SITES);
        let target_site = match sites.get(0) {
            Some(s) => s,
            None => return Ok(false)
        };

        if creep.build(target_site) == ReturnCode::NotInRange {
            creep.move_to(target_site);
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "build"
    }
}