use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep,
    memory::MemoryReference,
    constants::Part
};
use crate::traits::{Role, Task, FlagProcessor};
use crate::tasks::{
    harvest::TaskHarvest,
    refill::TaskRefill,
    build::TaskBuild,
    upgrade::TaskUpgrade,
};

/// A builder creep refills itself when empty, otherwise tries to do the following tasks in order:
///   1. `tasks/build`
///   2. `tasks/refill`
///   3. `tasks/upgrade`
pub struct Builder<'a> {
    memory: MemoryReference,
    harvest: &'a TaskHarvest,
    refill: &'a TaskRefill,
    build: &'a TaskBuild,
    upgrade: &'a TaskUpgrade
}

impl<'a> Builder<'a> {
    pub fn new(memory: MemoryReference, build: &'a TaskBuild, harvest: &'a TaskHarvest, refill: &'a TaskRefill, upgrade: &'a TaskUpgrade) -> Builder<'a> {
        memory.set("run_count", 0);
        
        Builder{
            memory: memory,
            harvest: harvest,
            refill: refill,
            build: build,
            upgrade: upgrade
        }
    }
}

impl<'a> FlagProcessor for Builder<'a> {}

impl<'a> Role for Builder<'a> {
    fn name(&self) -> &'static str {
        "builder"
    }

    fn limit(&self) -> i32 {
        2
    }

    fn next_creep(&self) -> Vec<Part> {
        vec![Part::Work, Part::Carry, Part::Move]
    }

    fn run_count(&self) -> i32 {
        self.memory.get("run_count").unwrap_or(0)
    }

    fn run(&self, creep: &Creep) -> Result<(), Box<dyn Error>> {
        self.memory.set("run_count", self.run_count() + 1);

        let harvesting = match creep.energy() {
            0 => true,
            carry if carry >= creep.carry_capacity() => false,
            _ => creep.memory().bool("harvesting")
        };

        creep.memory().set("harvesting", harvesting);

        if harvesting {
            self.harvest.run(creep)?;
            Ok(())
        } else {
            match true {
                _ if self.build.run(creep)?   => Ok(()),
                _ if self.refill.run(creep)?  => Ok(()),
                _ if self.upgrade.run(creep)? => Ok(()),

                _ => Err(Box::from("all of the tasks failed to run"))
            }
        }
    }

    fn spawn_priority(&self) -> i32 {
        15
    }
}