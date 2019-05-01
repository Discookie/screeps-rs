use std::error::Error;
use screeps::{
    prelude::*,
    objects::Creep
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
    harvest: &'a TaskHarvest,
    upgrade: &'a TaskUpgrade
}

impl<'a> Upgrader<'a> {
    pub fn new(harvest: &'a TaskHarvest, upgrade: &'a TaskUpgrade) -> Upgrader<'a> {
        Upgrader{
            harvest: harvest,
            upgrade: upgrade
        }
    }
}

impl<'a> Role for Upgrader<'a> {
    fn name(&self) -> &'static str {
        "upgrader"
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
                _ if self.upgrade.run(creep)? => Ok(()),

                _ => Err(Box::from("all of the tasks failed to run"))
            }
        }
    }
}