use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep
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
    harvest: &'a TaskHarvest,
    refill: &'a TaskRefill,
    build: &'a TaskBuild,
    upgrade: &'a TaskUpgrade
}

impl<'a> Harvester<'a> {
    pub fn new(build: &'a TaskBuild, harvest: &'a TaskHarvest, refill: &'a TaskRefill, upgrade: &'a TaskUpgrade) -> Harvester<'a> {
        Harvester{
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

    fn run(&self, creep: &Creep) -> Result<(), Box<Error>> {
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