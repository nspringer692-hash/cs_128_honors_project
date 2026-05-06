use bevy::{prelude::*};
use crate::gate::{Gate, GateType};
use std::collections::HashMap;

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
        let mut graphing = vec![vec![None; 8]; size];
        graphing[0][0] = Some(10001);
        Self {
            gates: Vec::with_capacity(size),
            graph: graphing,
            num_inputs,
        }
    }
    // to be used when a new gate is added to the "field", uses the new function from 
    // this also inputs a blank row and column to the empty matrix
    // this function also clearly assigns an id value to each newly created gate in the "playarea/field"
    pub fn add_gate(&mut self, gate_type: GateType) {

        let insert = Gate::new(gate_type);
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
        for i in 0..self.graph.len() {
            for j in 0..self.graph[i].len() {
                if self.graph[i][j] == Some(get_id as usize) {
                    self.graph[i][j] = None;
                }
            }
        }
    }

    // when two gates are connected, this function is to be used
    // will also delete connections when a gate is deleted that is also
    // connected with wires to other gates
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
    // helper for check_connection, just checks to seee if a value is in the row given with the id
    fn find_row(&self, node_id: usize) -> Option<&Vec<Option<usize>>> {
        for row in &self.graph {
            if row[0] == Some(node_id) {
                return Some(row);
            }
        }
        None
    }

    // helper that confirms that the output is only connected to the right amount of output values
    pub fn confirm_output(&self) -> bool {
        if &self.graph[0][1] == &None || &self.graph[0][2] != &None {
            return false;
        }
        true
    }

    // helper that confirms that all gates are connected and not dangling
    pub fn check_valid_connections(&self) -> bool {
        let mut correct = 0;
        for gate in 0..self.gates.len() {
            if self.gates[gate].kind == GateType::NOT {
                if self.graph[gate + 1][1] != None && self.graph[gate + 1][2] == None {
                    correct += 1;
                }
            } else {
                if self.graph[gate + 1][1] != None && self.graph[gate + 1][2] != None && self.graph[gate + 1][3] == None {
                    correct += 1;
                }
            }
        }
        return correct as usize == self.gates.len();
    }
    // checks to see if the input and the output are connected in some way, so that the user can test to see the
    // program output (with a DFS)
    pub fn check_connection(&mut self) -> bool {
        if self.confirm_output() == false {
            return false;
        }
        if self.check_valid_connections() == false {
            return false;
        }
        let mut to_check: Vec<usize> = vec![10001 as usize];
        let mut visited: Vec<usize> = Vec::new();
        let mut inputs = 0;
        while let Some(current) = to_check.pop() {
            // checking to see if the value is the input (id of 10000)
            if current == 10000 as usize || current == 10002 as usize {
                inputs += 1;
                if inputs == 2 {
                    return true;
                }
            }

            if visited.contains(&current) {
                continue;
            }
            visited.push(current);
            
            // get to the row that "current" has and iterate through it 
            if let Some(row) = self.find_row(current) {
                // this iteration basically loops through all the values that need to be searched through next
                for cell in row.iter().skip(1) {
                    if let Some(dep) = cell {
                        if !visited.contains(dep) {
                            to_check.push(*dep);
                        }
                    }
                }
            }
        }

        false

    }

    // essentially this function is the one that has the full logic of the gates and this is used to determine
    // what the output is when the input is put in as 0 or 1, using the logic for each gate to determine this
    pub fn evaluate(&self, input: bool, input2: bool) -> bool {
        let mut values: HashMap<usize, bool> = HashMap::new();
        values.insert(10000, input);
        values.insert(10002, input2);
        let mut queue: Vec<usize> = vec![10000, 10002];

        // essentially this function uses a sorta-topological to test the logic of the current gates
        //going throught the gate structure until reaching the output id, then returning that said id
        while !queue.is_empty() {
            let current = queue.pop().unwrap();

            for row in self.graph.iter() {
                // now the program is testing whether all values prior to this gate location have been tested
                // if any of the gates haven't been tested, then you can't continue
                if row.contains(&Some(current)) && row[0] != Some(current) {
                    let mut ready = true;
                    let mut i = 1;
                    while i < row.len() {
                        if let Some(dep_id) = row[i] {
                            if !values.contains_key(&dep_id) {
                                ready = false;
                                break;
                            }
                        }
                        i += 1;
                    }

                    if ready {
                        let mut inputs: Vec<bool> = Vec::new();
                        let mut i = 1;
                        // taking all values in that gate and storing them in inputs to be used later
                        while i < row.len() {
                            if let Some(dep_id) = row[i] {
                                inputs.push(*values.get(&dep_id).unwrap());
                            }
                            i += 1;
                        }
                        
                        let gate_id = row[0].unwrap();
                        // test to see whether the gate is the output
                        if gate_id == 10001 {
                            values.insert(gate_id, inputs[0]);
                            continue;
                        }

                        let mut found_gate = None;
                        for gate in &self.gates {
                            if gate.id as usize == gate_id {
                                found_gate = Some(gate);
                                break;
                            }
                        }

                        // from the lines, you are actually getting the actual output of the tested gates
                        // with the match statement, which will eventually get to the output value
                        let gate = found_gate.expect("Could not find a gate with that ID");
                        let result = match gate.kind {
                            GateType::AND  => inputs[0] && inputs[1],
                            GateType::OR   => inputs[0] || inputs[1],
                            GateType::NOT  => !inputs[0],
                            GateType::NAND => !(inputs[0] && inputs[1]),
                            GateType::NOR  => !(inputs[0] || inputs[1]),
                            GateType::XOR  => inputs[0] ^ inputs[1],
                            GateType::XNOR => !(inputs[0] ^ inputs[1]),
                        };
                        values.insert(gate_id, result);
                        queue.push(gate_id);
                    }
                }
            }
        }
        // from the values, this code will read what the value is from the adjacent value, in which it
        // is in the hashmap
        *values.get(&10001).unwrap_or(&false)
    }
}

// this is a wrapper, specifically used for the circuit struct and helps to distinguish
// the levels and which one is currently active
#[derive(Resource)]
pub struct ActiveCircuit(pub crate::circuit::Circuit);