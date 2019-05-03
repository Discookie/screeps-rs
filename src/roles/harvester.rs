use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep,
    memory::MemoryReference,
    constants::Part
};

use crate::traits::{Role, Task};
use crate::tasks::{
    harvest::TaskHarvest,
    refill::TaskRefill,
    build::TaskBuild,
    upgrade::TaskUpgrade,
};

/// A harvester creep refills itself when empty, otherwise tries to do the following tasks in order:
///   1. `tasks/refill`
///   2. `tasks/build`
///   3. `tasks/upgrade`
pub struct Harvester<'a> {
    memory: MemoryReference,
    harvest: &'a TaskHarvest,
    refill: &'a TaskRefill,
    build: &'a TaskBuild,
    upgrade: &'a TaskUpgrade
}

impl<'a> Harvester<'a> {
    pub fn new(memory: MemoryReference, build: &'a TaskBuild, harvest: &'a TaskHarvest, refill: &'a TaskRefill, upgrade: &'a TaskUpgrade) -> Harvester<'a> {
        memory.set("run_count", 0);
        
        Harvester{
            memory: memory,
            harvest: harvest,
            refill: refill,
            build: build,
            upgrade: upgrade
        }
    }
}

impl<'a> Role for Harvester<'a> {
    fn name(&self) -> &'static str {
        "harvester"
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

    fn run(&self, creep: &Creep) -> Result<(), Box<Error>> {
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
                _ if self.refill.run(creep)?  => Ok(()),
                _ if self.build.run(creep)?   => Ok(()),
                _ if self.upgrade.run(creep)? => Ok(()),

                _ => Err(Box::from("all of the tasks failed to run"))
            }
        }
    }
}