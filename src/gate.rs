use std::sync::{Mutex, OnceLock};
use bevy::prelude::*;


pub static GLOBAL_ID: Mutex<i32> = Mutex::new(0);


//all the type of gates
#[derive(Component)]
#[derive(Debug, Clone)]
pub enum GateType {
    NAND,
    NOR,
    AND,
    OR,
    NOT,
    XOR,
    XNOR,
}


//the Gate struct determines all the values in a logic gate, and determines what to do with each gate when utilized.

#[derive(Component)]
#[derive(Debug)]
pub struct Gate {
    pub kind: GateType,
    pub input_states: Vec<bool>,
    pub output: bool,
    pub id: i32,
}


impl Gate {
    //create a new gete, this can be used when a gate is dragged into the "field"
    pub fn new(kind: GateType, input_amount: usize) -> Self {
        let id = {
            let mut guard = GLOBAL_ID.lock().unwrap();
            let current = *guard;
            *guard += 1; // Increment for the next gate
            current
        };
        Self {
            kind,
            input_states: vec![false; input_amount],
            output: false,
            id,
        }
    }
}
