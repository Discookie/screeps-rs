use std::error::Error;
use screeps::{
    prelude::*,
    objects::{
        Creep,
        StructureController
    },
    constants::*,
    game::spawns
};
use crate::traits::Role;

pub struct Upgrader;

impl Role for Upgrader {
    fn name(&self) -> &'static str {
        "upgrader"
    }

    fn run(&mut self, creep: &Creep) -> Result<(), Box<Error>> {
        let harvesting = match creep.energy() {
            0 => true,
            carry if carry >= creep.carry_capacity() => false,
            _ => creep.memory().bool("harvesting")
        };

        if harvesting {
            let sources = creep.room().find(find::SOURCES);
            let target_src = sources.get(0).ok_or("there are no sources")?;

            if creep.harvest(target_src) == ReturnCode::NotInRange {
                creep.move_to(target_src);
            }
        } else {
            let controller: StructureController = creep.room().controller().ok_or("there is no controller")?;

            if creep.upgrade_controller(&controller) == ReturnCode::NotInRange {
                creep.move_to(&controller);
            }
        }

        creep.memory().set("harvesting", harvesting);

        Ok(())
    }
}