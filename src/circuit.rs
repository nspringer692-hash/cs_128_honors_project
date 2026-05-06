use bevy::{prelude::*};
use crate::gate::{Gate, GateType};

//this is the actual graph, with each gate being stated in the gates vector and the graph being the actual was to find run through the values
#[derive(Component)]
pub struct Circuit {
    pub gates: Vec<Gate>,
    pub graph: Vec<Vec<Option<usize>>>,
    pub num_inputs: u32,
}


impl Circuit {

    // create a new circuit, probably used for each level
    // this essentially stores all the information the level needs to know, the inputs, the connections, and if the level is passed or not
    pub fn new(num_inputs: u32, size: usize) -> Self {
        Self {
            gates: Vec::with_capacity(size),
            graph: vec![vec![None; size]; size],
            num_inputs,
        }
    }
    // to be used when a new gate is added to the "field", uses the new function from 
    // this also inputs a blank row and column to the empty matrix
    // this function also clearly assigns an id value to each newly created gate in the "playarea/field"
    pub fn add_gate(&mut self, gate_type: GateType) {
        let value: i32 = match gate_type {
            GateType::NOT => 1,
            _ => 2,
        };

        let insert = Gate::new(gate_type, value as usize);
        let curr_id = insert.id as usize;
        self.add_existing_gate(insert, curr_id);
    }
    
    // essentially a helper function for add_gate, just intiializes the null values to the graph
    pub fn add_existing_gate(&mut self, gate: Gate, id: usize) {
        self.gates.push(gate);

        let mut row = vec![None; 8];
        row[0] = Some(id);
        self.graph.push(row);
    }

    // this should be used when a gate is taken off the "field", and this is only possible when the gate is not attached to anything
    // this function will take away that value from the vector with the gates in it.
    // taking the values out of the graph is yet to be implemented
    // (tested, implementation correctly works, at lease with the vetor of gates)
    pub fn remove_gate(&mut self, get_id: i32) {
        // this section of gate is specifically used to remove the correct id from gates
        let mut index = 0;
        let mut found = false;
        for (i, value) in self.gates.iter().enumerate() {
            if value.id == get_id {
                index = i;
                found = true;
                break;
            }
        }
        if found {
            self.gates.remove(index);
        }

        // this section is responsible for the removal of the correct row in graph
        let row_id = get_id as usize;
        for i in 0..self.graph.len() {
            if self.graph[i][0] == Some(row_id) {
                self.graph.remove(i);
                break;
            }
        }
    }

    // when two gates are connected, this function is to be used (not tested yet)
    pub fn connect_gates(&mut self, from_id: usize, to_id: usize) {
        for i in 0..self.graph.len() {
            if self.graph[i][0] == Some(from_id) {
                for value in 0..self.graph[i].len() {
                    if self.graph[i][value] == None {
                        self.graph[i][value] = Some(to_id);
                        return;
                    }
                }
            }
        }
    }
}

// this is a wrapper, specifically used for the circuit struct and helps to distinguish
// the levels and which one is currently active
#[derive(Resource)]
pub struct ActiveCircuit(pub crate::circuit::Circuit);