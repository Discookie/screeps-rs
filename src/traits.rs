use screeps::objects::Creep;
use std::error::Error;

pub trait Role {
    fn name(&self) -> &'static str {
        "undefined"
    }

    fn run(&mut self, creep: &Creep) -> Result<(), Box<Error>>;
}