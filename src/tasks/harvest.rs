use std::{
    convert::From,
    error::Error
};
use screeps::{
    constants::*,
    game::get_object_typed,
    memory::{
        MemoryReference,
        root
    },
    objects::{
        Creep,
        RoomPosition,
        Source
    },
    prelude::*
};

use crate::traits::Task;

/// A creep moves to its assigned source, and begins harvesting.
/// 
/// Sources are stored in `memory.sources`.  
/// What's stored inside a source:
///   * `creep_limit` - how many creeps can harvest at the same time
///   * `counter` - how many creeps harvested in this tick
///   * `prev_counter` - how many creeps harvested in the previous tick
///   * `pos` - an {x, y, room} object that stores the source's position
///     * Populates when the first creep begins harvesting there.
/// 
/// A source cannot be removed, but its limit can be set to 0.
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
        // 
        let sources = task.memory.dict_or_create("sources").expect("failed to create task memory");

        for source_id in sources.keys() {
            let source = sources.dict(&source_id).unwrap().unwrap();
            source.set("prev_counter", source.i32("counter").unwrap_or(None).unwrap_or(0));
            source.set("counter", 0);
        }

        task
    }

    #[inline]
    fn add_to_counter(&self, id: &str) {
        if let Ok(source) = self.memory.dict_or_create("sources").and_then(|src| src.dict_or_create(id)) {
            source.set("counter", 1 + source.i32("counter").unwrap_or(None).unwrap_or(0));
        }
    }

    #[inline]
    fn add_to_prev_counter(&self, id: &str) {
        if let Ok(source) = self.memory.dict_or_create("sources").and_then(|src| src.dict_or_create(id)) {
            source.set("prev_counter", 1 + source.i32("prev_counter").unwrap_or(None).unwrap_or(0));
        }
    }

    #[inline]
    fn get_counter(&self, id: &str) -> i32 {
        if let Ok(source) = self.memory.dict_or_create("sources").and_then(|src| src.dict_or_create(id)) {
            source.i32("prev_counter").unwrap_or(None).unwrap_or(0)
        } else {
            0
        }
    }

    #[inline]
    fn get_limit(&self, id: &str) -> i32 {
        if let Ok(source) = self.memory.dict_or_create("sources").and_then(|src| src.dict_or_create(id)) {
            source.i32("creep_limit").unwrap_or(None).unwrap_or(4)
        } else {
            4
        }
    }


}

impl Task for TaskHarvest {
    fn run(&self, creep: &Creep) -> Result<bool, Box<dyn Error>> {
        let memory = self.creep_memory(creep)?;
        let sources = self.memory.dict_or_create("sources")?;

        let mut source_opt = { // reading stored target from memory
            match memory.string("source") {
                Ok(Some(id)) => match get_object_typed::<Source>(&id) {
                    Ok(Some(ref source)) if source.energy() == 0 => None,
                    _ => Some(id)
                },
                _ => None
            }
        };

        source_opt = match source_opt {
            None => { // selecting a new target
                let mut source_ids = sources.keys();
                source_ids.retain(|source_id| {
                    (match get_object_typed::<Source>(&source_id) {
                        Ok(Some(ref source)) if source.energy() == 0 => false,
                        _ => true
                    }) && self.get_counter(&source_id) < self.get_limit(&source_id)
                });

                match source_ids.len() {
                    0 => None,
                    len => {
                        let source_id = {
                            let id = creep.memory().i32("id").unwrap_or(None).unwrap_or(0) as usize;
                            source_ids[id % len].to_owned()
                        };

                        memory.set("source", &source_id);
                        self.add_to_prev_counter(&source_id);

                        Some(source_id)
                    }
                }
            },
            x => x
        };
        
        if let Some(source_id) = source_opt {
            self.add_to_counter(&source_id);
            memory.set("source", &source_id);

            if let Some(pos) = 
                sources.dict_or_create(&source_id)?.dict_or_create("pos").ok()
                .and_then(|pos_root| {
                    let x = pos_root.get("x").ok()?;
                    let y = pos_root.get("y").ok()?;
                    let room: String = pos_root.get("room").ok()?;

                    Some(RoomPosition::new(x, y, &room))
                }) 
            { // we have a stored position
                if creep.pos().is_near_to(&pos) {
                    match get_object_typed::<Source>(&source_id)? {
                        Some(source) => {
                            creep.harvest(&source);
                        },
                        None => {
                            warn!("Invalid source: {} - bad pos, in a different room", source_id);
                        }
                    }
                } else {
                    creep.move_to(&pos);
                }
            } else { // we do not have a stored position
                match get_object_typed::<Source>(&source_id)? {
                    Some(source) => {
                        if creep.pos().is_near_to(&source) {
                            creep.harvest(&source);
                        } else {
                            creep.move_to(&source);
                        }
                        
                        // write the position
                        let source_pos = source.pos();
                        let pos_root = sources.dict_or_create(&source_id)?.dict_or_create("pos")?;
                        pos_root.set("x", source_pos.x());
                        pos_root.set("y", source_pos.y());
                        pos_root.set("room", source_pos.room_name());
                    },
                    None => {
                        warn!("Invalid source: {} - missing pos", source_id);
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "harvest"
    }

    fn memory(&self) -> Result<MemoryReference, Box<dyn Error>> {
        Ok(self.memory.clone())
    }
}