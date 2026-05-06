/*
# Ferroforge - a Rust-based circuit game

# Project introduction:
    - For our final project, we will be making a point-and-click puzzle game revolving around solving basic computer architecture logic puzzles. 
    - The user will traverse several levels, ranging from constructing the seven fundamental logic gates to creating fundamental logic systems.
    - The ultimate goal is to implement various pieces of computer hardware from the ground up.

# Technical Overview
    - The user interacts with a custom UI and navigates various levels
    - Custom UI powered by bevy_egui and ECS layout powered by bevy
    - The user starts with developing the basic gates starting from the humble NAND gate

# References:
    - ECE120 (frying Daniel's brain)
    - Turing Complete 
    - NandGame
*/

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy::{input_focus::InputFocus};

// All external files in src
pub mod gate;
pub mod circuit;
pub mod ui;
pub mod textures;
pub mod components;
pub mod block;

// Get the functions and structs from each file
// use gate::{Gate, GateType}; Unused stuff for now
// use circuit::Circuit;
use ui::*; // Access all ui.rs functions
use textures::*; // Access all textures.rs functions
use components::*; // Access all components.rs functions

// Get the backend functions
use circuit::*;
// use block::*;

// Overall startup, creating the app, running throught the assets and running the program.
fn main() {
    App::new() // Create new app
    .insert_resource(ActiveCircuit(crate::circuit::Circuit::new(0, 0)))
    .insert_resource(DragState::default()) // Create new global resource to track drag state
    .insert_resource(PopupState::default()) // Create new global resource for tracking popup
    .insert_resource(CurrentStat {
        input: false,
        working_output: false,
        output: -1,
    })
    .insert_resource(ConnectionState::default()) // Create new global resource for the current
    .add_plugins(DefaultPlugins) // Plugins for Bevy game development
    .add_plugins(EguiPlugin::default()) // Plugins for Bevy egui
    .init_state::<GameState>() // Set initial game state
    .init_resource::<InputFocus>()
    .add_systems(Startup, load_textures) 
    .add_systems(Startup, setup.after(load_textures)) // Run setup process once
    .add_systems(Update, button_system)
    .add_systems(EguiPrimaryContextPass, user_interface) // Load user interface
    .add_systems(Update, ( // Run certain functions once per frame / every 60 secs
        start_drag_system,
        drag_system,
        end_drag_system,
        handle_spawn_gate,
        delete_on_right_click,
        process_circuit_simulation,
        select_input_port,
        connect_to_output.after(select_input_port),
        update_wires,
    ))
    .add_message::<SpawnGateEvent>()
    .run();
}

//used in setting up the system *
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(button(&asset_server, 450.0, 320.0, 125, 60));
    spawn_grid(&mut commands);
}

fn process_circuit_simulation(mut active_circuit: ResMut<ActiveCircuit>) {

    // Currently unused
    let _circuit = &mut active_circuit.0;
}