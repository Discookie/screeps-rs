#![recursion_limit = "128"]
#![feature(drain_filter)]
extern crate fern;
extern crate hashbrown;
#[macro_use]
extern crate log;
extern crate screeps;
#[macro_use]
extern crate stdweb;

mod logging;
mod military;
mod roles;
mod tasks;
mod traits;

use hashbrown::HashMap;

use screeps::{
    constants::{
        ReturnCode,
        Part
    },
    prelude::*,
    objects::*,
    game::*,
    memory::{root, MemoryReference}
};

use crate::{
    military::tower::Tower,
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
                    @{stdweb::initialize()}
                }
            }
        }
    }
}

fn game_loop() {
    let mut err_counter = 0;

    // Military tasks
    let military_root = root().dict_or_create("military").unwrap();
    let tower_root = military_root.dict_or_create("tower").unwrap();

    let tower_handler = Tower::new(tower_root);
    
    tower_handler.run().unwrap_or_else(|err| {
            warn!("failed to execute tower handler: {}", err.to_string());
            err_counter += 1;
        });


    // Creep tasks
    let _task_root = root().dict_or_create("tasks").unwrap();
    let role_root = root().dict_or_create("roles").unwrap();

        
    let task_build = TaskBuild::new();
    let task_harvest = TaskHarvest::new();
    let task_refill = TaskRefill::new();
    let task_upgrade = TaskUpgrade::new();

    let roles = {
        let mut map: HashMap<&str, Box<dyn Role>> = HashMap::new();

        // TODO clean this up and use HashMaps or some better resolver
        let role_builder = Builder::new(role_root.dict_or_create("builder").unwrap(),
                                       &task_build, &task_harvest, &task_refill, &task_upgrade);
        let role_harvester = Harvester::new(role_root.dict_or_create("harvester").unwrap(),
                                           &task_build, &task_harvest, &task_refill, &task_upgrade);
        let role_upgrader = Upgrader::new(role_root.dict_or_create("upgrader").unwrap(),
                                         &task_harvest, &task_upgrade);

        map.insert(role_builder.name(), Box::from(role_builder));
        map.insert(role_harvester.name(), Box::from(role_harvester));
        map.insert(role_upgrader.name(), Box::from(role_upgrader));

        map
    };

    

    for creep in creeps::values() {
        let role_str = creep.memory().string("role").unwrap_or( Some("error".to_string()) ).unwrap_or( "missing".to_string() );
        if let Err(err) = roles.get(role_str.as_str()).ok_or(Box::from(format!("unknown role {}", role_str)))
                            .and_then(|role| role.run(&creep)) {
            warn!("failed to execute task for creep {}: {}", creep.name(), err.to_string());
            err_counter += 1;
        }
    }

    // New creep creation
    if let Some(spawn) = spawns::get("Spawn1") {
        fn next_id() -> i32 {
            let id = root().i32("id").unwrap_or(None).unwrap_or(0);
            id
        }

        fn step_id() {
            root().set("id", next_id()+1);
        }

        fn make_mem(role: &str, id: i32) -> MemoryReference {
            let reference = MemoryReference::new();
            reference.set("role", role);
            reference.set("id", id);
            reference
        }

        fn spawn_creep(spawn: &StructureSpawn, body: &Vec<Part>, id: &i32, options: &SpawnOptions) -> bool {
            if spawn.spawn_creep_with_options(body.as_slice(), &id.to_string(), options) == ReturnCode::Ok {
                step_id();
                true
            } else {
                false
            }
        }

        if !spawn.is_spawning() {
            for (role_str, role) in roles.iter() {
                let body = role.next_creep();
                let id = next_id();
                let options = SpawnOptions::new()
                                .memory( make_mem(role.name(), id) );

                if spawn_creep(&spawn, &body, &id, &options) {
                    break;
                }
            }
        }
    }

    if time() % 50 == 3 {
        clear_deceased();
        
        info!("creep counts: <ul>{}</ul>", roles.iter().fold(String::new(), |acc, (role_str, role)| {
                format!("{acc}<li>{name}: {run}/{limit}</li>", acc=acc, name=role_str, run=role.run_count(), limit=role.limit())
            }
        ));
    }

    if err_counter > 10 {
        error!("too many errors, sending notification");
        notify("Encountered too many errors!", None);
    }
}

fn clear_deceased() {
    let creep_directory = root().dict_or_create("creeps").expect("Memory.creeps is not a dict");
    for name in creep_directory.keys() {
        if let None = creeps::get(&name) {
            creep_directory.del(&name);
        }
    }
}
