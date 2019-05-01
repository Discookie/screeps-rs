use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep,
    constants::*
};

use crate::traits::Task;

/// A creep moves to its assigned source, and begins harvesting.
pub struct TaskHarvest;

impl TaskHarvest {
    pub fn new() -> TaskHarvest {
        TaskHarvest{}
    }
}

impl Task for TaskHarvest {
    fn run(&self, creep: &Creep) -> Result<bool, Box<Error>> {
        let mut sources = creep.room().find(find::SOURCES);
        let mut filt = sources.drain_filter(|source| {
            source.energy() > 0
        });
        
        if let Some(target_src) = filt.next() {
            if creep.harvest(&target_src) == ReturnCode::NotInRange {
                creep.move_to(&target_src);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}