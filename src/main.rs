#![recursion_limit = "128"]
extern crate fern;
#[macro_use]
extern crate log;
extern crate screeps;
#[macro_use]
extern crate stdweb;

mod logging;
mod traits;
mod roles;

use std::error::Error;
use screeps::{
    prelude::*,
    objects::*,
    game::*
};

use crate::{
    traits::Role,
    roles::{
        harvester::Harvester,
        upgrader::Upgrader
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
    let mut role_harvester = Harvester{};
    let mut role_upgrader = Upgrader{};
    let mut err_counter = 0;

    for creep in creeps::values() {

        match &creep.memory().string("role").unwrap_or( Some("error".to_string()) ).unwrap_or( "missing".to_string() )
        {
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
