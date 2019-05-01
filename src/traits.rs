use screeps::objects::Creep;
use std::error::Error;

pub trait Role {
    fn name(&self) -> String {
        "undefined".to_string()
    }

    fn run(&mut self, creep: &Creep) -> Result<(), Box<Error>>;
}