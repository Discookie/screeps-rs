use std::error::Error;

use screeps::{
    constants::find,
    memory::MemoryReference,
    objects::{
        Attackable,
        Creep,
        Structure,
        StructureTower
    },
    prelude::*,
    game
};

/// Handles all towers.
/// Each added tower has a separate target stored in memory.
/// Add a new tower by creating a dict inside `memory`, with the tower's id as its name.
pub struct Tower {
    memory: MemoryReference
}

impl Tower {
    pub fn new(memory: MemoryReference) -> Tower {
        Tower{
            memory: memory
        }
    }

    fn get_target(&self, memory: &MemoryReference) -> Option<Creep> {
        memory.string("target").unwrap_or(None)
            .and_then( |target_id| game::get_object_typed(&target_id).unwrap_or(None) )
    }

    fn new_target(&self, tower: &StructureTower) -> Option<Creep> {
        let target = tower.pos().find_closest_by_range(find::HOSTILE_CREEPS);
        target
    }

    fn get_job(&self, memory: &MemoryReference) -> Option<Structure> {
        memory.string("job").unwrap_or(None)
            .and_then( |job_id| game::get_object_typed(&job_id).unwrap_or(None) )
            .filter( |job: &Structure| job.as_attackable().map(|x| x.hits() < x.hits_max()).unwrap_or(false) )
    }

    fn new_job(&self, tower: &StructureTower) -> Option<Structure> {
        let mut targets: Vec<Structure> = tower.room().find(find::STRUCTURES);

        for target in targets.drain(..) {
            if target.as_attackable().map(|x| x.hits() < x.hits_max()).unwrap_or(false) {
                return Some(target);
            }
        }
        None
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let tower_list = self.memory.dict_or_create("towers").map_err(|x| x.to_string())?;

        for tower_id in tower_list.keys() {
            if let Ok(Some(tower)) = game::get_object_typed(&tower_id) {
                let dict = tower_list.dict_or_create(&tower_id).map_err(|x| x.to_string())?;
                self.run_tower(&tower, &dict)?;
            } else {
                warn!("Deleting nonexistent tower {}", tower_id);
                tower_list.del(&tower_id);
            }
        }

        Ok(())
    }

    pub fn run_tower(&self, tower: &StructureTower, memory: &MemoryReference) -> Result<(), Box<dyn Error>> {
        if let Some(target) = self.get_target(&memory).or_else(|| self.new_target(&tower)) {
            memory.set("target", target.id());
            tower.attack(&target);
        } else {
            memory.del("target");
        }
        
        if let Some(job) = self.get_job(&memory).or_else(|| self.new_job(&tower)) {
            memory.set("job", job.id());
            tower.repair(&job);
        } else {
            memory.del("job");
        }
        
        Ok(())
    }
}