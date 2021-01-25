// Stores the config for each session;

use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Config_Handler {
    pub payload: String
}

pub fn new() -> Config_Handler {
    Config_Handler {
        payload: String::from("PING")
    }
}