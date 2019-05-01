use std::error::Error;
use screeps::{
    prelude::*,
    objects::{
        Creep,
        StructureSpawn
    },
    constants::*,
    game::spawns
};
use crate::traits::Role;

pub struct Harvester;

impl Role for Harvester {
    fn name(&self) -> &'static str {
        "harvester"
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
            let spawn: StructureSpawn = spawns::get("Spawn1").ok_or("spawn does not exist")?;

            if creep.transfer_all(&spawn, ResourceType::Energy) == ReturnCode::NotInRange {
                creep.move_to(&spawn);
            }
        }

        creep.memory().set("harvesting", harvesting);

        Ok(())
    }
}