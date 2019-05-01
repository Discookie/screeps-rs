#![recursion_limit = "128"]
#![feature(drain_filter)]
extern crate fern;
#[macro_use]
extern crate log;
extern crate screeps;
#[macro_use]
extern crate stdweb;

mod logging;
mod roles;
mod tasks;
mod traits;

use screeps::{
    prelude::*,
    objects::*,
    game::*
};

use crate::{
    traits::Role,
    roles::{
        builder::Builder,
        harvester::Harvester,
        upgrader::Upgrader
    },
    tasks::{
        build::TaskBuild,
        harvest::TaskHarvest,
        refill::TaskRefill,
        upgrade::TaskUpgrade
    }
};

fn main() {
    stdweb::initialize();
    logging::setup_logging(logging::Info);

    js! {
        var game_loop = @{game_loop};

        module.exports.loop = function() {
            // Provide actual error traces.
            try {
                game_loop();
            } catch (error) {
                // console_error function provided by 'screeps-game-api'
                console_error("caught exception:", error);
                if (error.stack) {
                    console_error("stack trace:", error.stack);
                }
                console_error("resetting VM next tick.");
                // reset the VM since we don't know if everything was cleaned up and don't
                // want an inconsistent state.
                module.exports.loop = function() {
                    __initialize(new WebAssembly.Module(require("compiled")), false);
                    module.exports.loop();
                }
            }
        }
    }
}

fn game_loop() {
    let task_build = TaskBuild::new();
    let task_harvest = TaskHarvest::new();
    let task_refill = TaskRefill::new();
    let task_upgrade = TaskUpgrade::new();

    let role_builder = Builder::new(&task_build, &task_harvest, &task_refill, &task_upgrade);
    let role_harvester = Harvester::new(&task_build, &task_harvest, &task_refill, &task_upgrade);
    let role_upgrader = Upgrader::new(&task_harvest, &task_upgrade);
    let mut err_counter = 0;

    for creep in creeps::values() {

        match &creep.memory().string("role").unwrap_or( Some("error".to_string()) ).unwrap_or( "missing".to_string() )
        {
            r if r == role_builder.name() => role_builder.run(&creep),
            r if r == role_harvester.name() => role_harvester.run(&creep),
            r if r == role_upgrader.name() => role_upgrader.run(&creep),
            role => Err(Box::from(format!("unknown role {}", role)))
        }.unwrap_or_else(|err| {
            warn!("failed to execute task for creep {}: {}", creep.name(), err.to_string());
            err_counter += 1;
        });
    }

    if err_counter > 10 {
        error!("too many errors, sending notification");
        notify("Encountered too many errors!", None);
    }
}
