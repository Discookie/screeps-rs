use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep,
    constants::*,
};

use crate::traits::Task;

/// A creep moves to the next empty object, and refills it.
pub struct TaskRefill;

impl TaskRefill {
    pub fn new() -> TaskRefill {
        TaskRefill{}
    }
}

impl Task for TaskRefill {
    fn run(&self, creep: &Creep) -> Result<bool, Box<Error>> {
        let mut targets = creep.room().find(find::STRUCTURES);
        let mut filt = targets.drain_filter(|structure| {
            match structure.as_can_store_energy() {
                Some(structure) => structure.energy() < structure.energy_capacity(),
                None => false
            }
            
        });
        
        if let Some(target_src) = filt.next() {
            if creep.transfer_all(target_src.as_transferable().unwrap(), ResourceType::Energy) == ReturnCode::NotInRange {
                creep.move_to(&target_src);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "refill"
    }
}