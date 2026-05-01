/*
# Ferroforge - a Rust-based circuit game

#
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

// Get the functions and structs from each file
// use gate::{Gate, GateType}; Unused stuff for now
// use circuit::Circuit;
use ui::*; // Access all ui.rs functions
use textures::*; // Access all textures.rs functions
use components::*; // Access all components.rs functions

// Overall startup, creating the app, running throught the assets and running the program.
fn main() {
    App::new() // Create new app
    .insert_resource(DragState::default()) // Create new global resource to track drag state
    .insert_resource(PopupState::default()) // Create new global resource for tracking popup
    .insert_resource(CurrentStat {
        input: false,
        working_output: false,
        output: -1,
    }) // Create new global resource for the current
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
    ))
    .add_message::<SpawnGateEvent>()
    .run();
}

// creates the texture of the gates themselves, while using nand.png. Setting these objects
// in the set coords, for example Vec3::new(-100.0, 0.0, 0.0) is put in the set coords given.

//used in setting up the system *
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, textures: Res<Textures>) {
    commands.spawn(Camera2d);
    commands.spawn(button(&asset_server, 450.0, 320.0, 125, 60));
    spawn_grid(&mut commands);

    spawn_block(&mut commands, Vec3::new(-100.0, 0.0, 0.0), &textures); // Green
    spawn_block(&mut commands, Vec3::new(100.0, 0.0, 0.0), &textures); // Red
    spawn_block(&mut commands, Vec3::new(0.0, 100.0, 0.0), &textures); // Blue
}
//      ^
//      |
//      |
// Spawn custom objects



//logic gate (placeholder functions)
// fn not_gate(input: bool) -> Output {
//     Output { out: !input }
// }


// fn int_and_out(input: Inputs, gate: GateType) -> Output {
//     match gate {
//         GateType::AND => return Output { out: input.in_a && input.in_b },
//         GateType::NAND => return Output { out: !input.in_a || !input.in_b },
//         GateType::NOR => return Output { out: !input.in_a && !input.in_b },
//         GateType::OR => return Output { out: input.in_a || input.in_b },
//         GateType::XNOR => return Output { out: input.in_a == input.in_b },
//         GateType::XOR => return Output { out: input.in_a != input.in_b },
//         _ => panic!("not including NOT"),
//     }
// }


