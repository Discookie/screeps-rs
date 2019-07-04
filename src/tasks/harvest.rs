use std::error::Error;
use screeps::{
    constants::*,
    game::get_object_typed,
    memory::{
        MemoryReference,
        root
    },
    objects::{
        Creep,
        Source
    },
    prelude::*
};

use crate::traits::Task;

/// A creep moves to its assigned source, and begins harvesting.
pub struct TaskHarvest {
    memory: MemoryReference
}

impl TaskHarvest {
    pub fn new() -> TaskHarvest {
        let memory = 
            root().dict_or_create("tasks")
                  .and_then(|mem| mem.dict_or_create("harvest"))
              .expect("failed to create task memory");
        let task = TaskHarvest{memory};

        // Copy last tick's counter
        task.memory.del("prev_counter");

        let counter = task.memory.dict_or_create("counter").expect("failed to create task memory");
        let prev_counter = task.memory.dict_or_create("prev_counter").expect("failed to create task memory");

        for key in counter.keys() {
            prev_counter.set(&key, counter.i32(&key).unwrap_or(None).unwrap_or(0));
            counter.set(&key, 0);
        }

        task
    }

    fn add_to_counter(&self, id: &str) {
        if let Ok(counter) = self.memory.dict_or_create("counter") {
            counter.set(id, 1 + counter.i32(id).unwrap_or(None).unwrap_or(0));
        }
    }

    fn add_to_prev_counter(&self, id: &str) {
        if let Ok(counter) = self.memory.dict_or_create("prev_counter") {
            counter.set(id, 1 + counter.i32(id).unwrap_or(None).unwrap_or(0));
        }
    }

    fn get_counter(&self, id: &str) -> i32 {
        if let Ok(counter) = self.memory.dict_or_create("prev_counter") {
            counter.i32(id).unwrap_or(None).unwrap_or(0)
        } else {
            0
        }
    }

    fn get_limit(&self, id: &str) -> i32 {
        if let Ok(counter) = self.memory.dict_or_create("creep_limits") {
            counter.i32(id).unwrap_or(None).unwrap_or(4)
        } else {
            4
        }
    }
}

impl Task for TaskHarvest {
    fn run(&self, creep: &Creep) -> Result<bool, Box<Error>> {
        let memory = self.creep_memory(creep)?;

        let mut source_opt = { // reading stored target from memory
            match memory.string("source") {
                Ok(Some(id)) =>
                    get_object_typed::<Source>(&id)
                        .unwrap_or(None)
                        .and_then(|source| {
                            match source.energy() {
                                0 => None,
                                _ => {
                                    Some(source)
                                }
                            }
                        }),

                _ => None
            }
        };

        source_opt = match source_opt {
            None => { // selecting a new target
                let mut sources = creep.room().find(find::SOURCES);
                sources.retain(|source| {
                    source.energy() > 0
                     && self.get_counter(&source.id()) < self.get_limit(&source.id())
                });

                match sources.len() {
                    0 => None,
                    len => {
                        let source = {
                            let id = creep.memory().i32("id").unwrap_or(None).unwrap_or(0) as usize;
                            sources[id % len].to_owned()
                        };

                        memory.set("source", source.id());
                        self.add_to_prev_counter(&source.id());

                        Some(source)
                    }
                }
            },
            x => x
        };
        
        if let Some(source) = source_opt {
            self.add_to_counter(&source.id());
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

    fn memory(&self) -> Result<MemoryReference, Box<Error>> {
        Ok(self.memory.clone())
    }
}