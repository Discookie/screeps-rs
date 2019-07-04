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
    upgrade::TaskUpgrade,
};

/// An upgrader creep either:
///   * Refills itself, or
///   * Upgrades the controller
pub struct Upgrader<'a> {
    memory: MemoryReference,
    harvest: &'a TaskHarvest,
    upgrade: &'a TaskUpgrade
}

impl<'a> Upgrader<'a> {
    pub fn new(memory: MemoryReference, harvest: &'a TaskHarvest, upgrade: &'a TaskUpgrade) -> Upgrader<'a> {
        memory.set("run_count", 0);

        Upgrader{
            memory: memory,
            harvest: harvest,
            upgrade: upgrade
        }
    }
}

impl<'a> Role for Upgrader<'a> {
    fn name(&self) -> &'static str {
        "upgrader"
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
                _ if self.upgrade.run(creep)? => Ok(()),

                _ => Err(Box::from("all of the tasks failed to run"))
            }
        }
    }

    fn spawn_priority(&self) -> i32 {
        15
    }
}