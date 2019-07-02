use std::error::Error;
use screeps::{
    game::get_object_typed,
    prelude::*,
    objects::{
        Creep,
        Source
    },
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
        let memory = self.creep_memory(creep)?;

        let source_opt = { // reading stored target from memory
            match memory.string("source") {
                Ok(Some(id)) =>
                    get_object_typed::<Source>(&id)
                        .unwrap_or(None)
                        .and_then(|source| match source.energy() {
                            0 => None,
                            _ => Some(source)
                        }),

                _ => None
            }
        }
        .or({ // selecting a new target
            let mut sources = creep.room().find(find::SOURCES);
            sources.retain(|source| {
                source.energy() > 0
            });
            
            match sources.len() {
                0 => None,
                len => {
                    let source = {
                        let id = creep.memory().i32("id").unwrap_or(None).unwrap_or(0) as usize;
                        sources[id % len].to_owned()
                    };
                    memory.set("source", source.id());

                    Some(source)
                }
            }
        });
        
        if let Some(source) = source_opt {
            if creep.harvest(&source) == ReturnCode::NotInRange {
                creep.move_to(&source);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "harvest"
    }
}