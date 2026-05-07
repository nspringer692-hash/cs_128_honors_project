use bevy::color::palettes::css::SANDY_BROWN;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::{input_focus::InputFocus};

use crate::gate::*;
use crate::textures::*;
use crate::components::*;
use crate::block::*;
use crate::circuit::*;

// Declare colors to represent the state of the CurrentStat button
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);


// Simple test resource : output 1 or 0 on button click in Editor state
#[derive(Resource)]
pub struct CurrentStat {
    pub input: bool,
    pub working_output: bool,
    pub output: i32,
}

// Pass a message to the gate spawning function and spawn a certain function depending on gate_type
#[derive(Message)]
pub struct SpawnGateEvent {
    position: Vec3,
    gate_type: GateType,
}

// List of game states to track for UI transitions
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Editor,
    Credits,
    LevelSelect,
}

// Set the grid size to 16
// Used for snapping to grid
const GRID_SIZE: f32 = 16.0;

/* 
    Bevy runs on an ECS system (entity component system)
    This is important to know since function parameters are basically:
    "Give me access to this data when the system runs"

    
    Entities are IDs representing objects in the world (gates, ports, wires)
    Components are data attached to entities
    Systems are functions that operate on data
    Resources are global resources that can be accessed by any system in the app

*/


/// bevy egui documentation found at https://github.com/vladbat00/bevy_egui

/// spawn_grid: spawn a series of grey lines across the play space every 16 coordinate points x and y

/// https://docs.rs/bevy/latest/bevy/prelude/struct.Commands.html
/// Referenced here to understand how to spawn and delete objects
/// 
/// # Arguments
/// 'commands' - Modify world by spawning entities
pub fn spawn_grid(commands: &mut Commands) {
    let spacing = 16.0; // Create a line every 16 units
    let half_size = 2000.0; // Stop creating lines at this point

    // Start from the left and add lines until x >= 0
    let mut x = -half_size;
    while x <= half_size {
        commands.spawn((
            // Create a grey line that spans the entire window
            Sprite {
                color: Color::srgba(0.4, 0.4, 0.4, 0.3),
                custom_size: Some(Vec2::new(1.0, half_size * 2.0)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, -10.0),
        ));
        x += spacing;
    }

    // Start from the bottom and add lines until y >= 0
    let mut y = -half_size;
    while y <= half_size {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.4, 0.4, 0.3),
                custom_size: Some(Vec2::new(half_size * 2.0, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, -10.0),
        ));
        y += spacing;
    }
}

/// Delete a gate and connected wires when the user right-clicks near it.
/// Only delete one entity per click

/// https://docs.rs/bevy/latest/bevy/prelude/struct.Transform.html
/// Referenced here to access the distance between an entity and a given point
/// 
/// https://bevy-cheatbook.github.io/input/mouse.html
/// Referenced here to figure out how to access cursor clicks
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'mouse' - Read the mouse input
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all entities with Transform, GateId components, and are Draggable
/// 'wires' - Access all entities with a Wire component
/// 'active_circuit' - Represents the backend for deleting the node associated with that gate
pub fn delete_on_right_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Transform, &GateId), With<Draggable>>,
    mut active_circuit: ResMut<ActiveCircuit>,
) {
    // If mouse is not right clicking, ignore
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    // Get cursor position
    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    // Loop through each entity in the world and find its distance from mouse
    for (entity, transform, gate_id) in &query {
        // Get the distance from the entity
        let dist = transform.translation.truncate().distance(cursor_pos);

        // If this entity is 40 units or less away from the mouse (hovered on), delete
        if dist < 40.0 {       
            // Delete the gate and its children
            commands.entity(entity).despawn();

            // Remove this gate from the graph
            let idx = active_circuit.active;
            active_circuit.circuits[idx].remove_gate(gate_id.0);
            break; // We only need to delete one!
        }
    }
}

/// Helper: cursor_to_world
/// cursor_to_world: Get the cursor's position, used for detecting which object is clicked
/// 
/// # Arguments
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
pub fn cursor_to_world(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let Ok(window) = windows.single() else {
        return None;
    };

    let cursor = window.cursor_position()?;

    let Ok((camera, cam_transform)) = cameras.single() else {
        return None;
    };

    camera.viewport_to_world_2d(cam_transform, cursor).ok()
}

/// start_drag_system: Detect when the left mouse button is clicked over an entity, then set DragState
/// 
/// # Arguments
/// 'drag_state' - Modify the current drag state so other drag functions know when to run
/// 'mouse' - Read the mouse input
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all draggable entities
pub fn start_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Transform), With<Draggable>>,
) {
    // Do not run this function if the left mouse button is lifted
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the cursor position
    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    // Check every draggable entity
    for (entity, transform) in &query {
        let dist = transform.translation.truncate().distance(cursor_pos);

        // If the mouse is hovering over the entity, set the drag state and exit
        if dist < 40.0 {
            drag_state.entity = Some(entity);
            break;
        }
    }
}

/// drag_system: Update this draggable entity's coordinates to be where the mouse is
/// 
/// # Arguments
/// 'drag_state' - Modify the current drag state
/// 'mouse' - Read the mouse input
/// 'windows' - Read over all windows to get cursor position on screen, needed by cursor_to_world
/// 'cameras' - Access screen coordinates, needed by cursor_to_world
/// 'query' - Access all draggable entities
pub fn drag_system(
    drag_state: Res<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform>,
) {
    // Get the current entity
    let Some(entity) = drag_state.entity else {
        return;
    };

    // If LMB is let go, do not run this function anymore
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    // Get cursor position
    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    // Transform this entity's position
    // Don't forget to snap to grid!
    if let Ok(mut transform) = query.get_mut(entity) {
        transform.translation.x = snap_to_grid(cursor_pos.x);
        transform.translation.y = snap_to_grid(cursor_pos.y);
    }
}

/// snap_to_grid: Force the passed floating value to be the value closest to it in a 16 coordinate grid
pub fn snap_to_grid(value: f32) -> f32 {
    // Divide the coordinate by the grid size
    // Round to nearest integer
    // Multiply back by grid size to get snapped position
    (value / GRID_SIZE).round() * GRID_SIZE
}

/// end_drag_system: Set the current drag state's entity to be None when the left mouse button is released
/// 
/// # Arguments
/// 'drag_state' - Set entity in DragState to None
/// 'mouse' - Read the mouse input
pub fn end_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.entity = None;
    }
}

/// The beefy one
/// user_interface: Handle windows and transitions and handle pressing buttons
/// 
/// Reference: https://github.com/vladbat00/bevy_egui
/// This has a list of tutorials on how to use various elements of bevy_egui
/// We used as reference many functions from the example WASM at https://vladbat00.github.io/bevy_egui/ui/
/// 
/// # Arguments
/// 'contexts' - Give access to egui to draw UI
/// 'state' - Read what state the game is currently in
/// 'next_state' - Update what state to change to next frame
/// 'message_writer' - Create a message for the handle_spawn_gate to read to determine what gate to spawn
/// 'popup' - Handle whether the help and tutorial popup should exist or not
pub fn user_interface(
    mut commands: Commands,  
    mut contexts: EguiContexts, // Give access to egui to draw UI
    state: Res<State<GameState>>, // Read what state the game is currently in
    mut next_state: ResMut<NextState<GameState>>, // What state to change to next frame?
    mut message_writer: MessageWriter<SpawnGateEvent>,
    mut active_circuit: ResMut<ActiveCircuit>,
    gate_query: Query<Entity, With<GateId>>,
    wire_query: Query<Entity, With<Wire>>,
    preview: Res<CircuitPreview>,
) -> Result {
    let ctx = contexts.ctx_mut()?; // Get access to bevy_egui's internal state
    match state.get() { // Depending on current state, show a certain window's contents
        GameState::MainMenu => { // If main menu, show main menu -> transition to other pages
            egui::CentralPanel::default().show(ctx, |ui| {
                // LET'S MAKE THIS STUFF BEAUTIFUL
                ui.label(
                    egui::RichText::new("Ferroforge")
                    .size(64.0)
                    .strong()
                ); // Set label for the window

                ui.separator(); // Add strikethrough border

                if ui // If Start Editor button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level Select"))
                    .clicked()
                {
                    next_state.set(GameState::LevelSelect);
                }

                if ui // If Credits button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Credits"))
                    .clicked()
                {
                    next_state.set(GameState::Credits);
                }

                if ui // If Quit button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Quit"))
                    .clicked()
                {
                    std::process::exit(0);
                }
            });
        }

        GameState::LevelSelect => {
            egui::CentralPanel::default().show(ctx, |ui| {
                // LET'S MAKE THIS STUFF BEAUTIFUL
                ui.label(
                    egui::RichText::new("Level Select")
                    .size(64.0)
                    .strong()
                );

                ui.separator(); // Add strikethrough border

                if ui // If Start Editor button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level 1"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
                }

                if ui // If Credits button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level 2"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
                }

                if ui // If Quit button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level 3"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
                }

                if ui // If Quit button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level 4"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
                }

                if ui // If Quit button pushed
                    .add_sized([250.0, 80.0], egui::Button::new("Level 5"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
                }
            });
        }

        GameState::Editor => { // If editor, show editor
            let all_gates: Vec<Entity> = gate_query.iter().collect();
            let all_wires: Vec<Entity> = wire_query.iter().collect();

            egui::SidePanel::left("Panel").show(ctx, |ui| {
                if ui.button("Back to Menu").clicked() { // Go back to main menu
                    next_state.set(GameState::MainMenu);
                }
                

                ui.label("Editor Mode"); // Set header as Editor Mode

                // Create space between label to create function preview
                ui.allocate_space(egui::Vec2::new(1.0, 100.0));

                ui.label("Level Instructions:");
                ui.label("Create an AND gate using only NAND and NOT.");

                ui.label("The truth table for AND is:");
                ui.label("A  B  |  F");
                ui.label("-----------");
                ui.label("0  0  |  0");
                ui.label("0  1  |  0");
                ui.label("1  0  |  0");
                ui.label("1  1  |  1");

                ui.allocate_space(egui::Vec2::new(1.0, 100.0));

                ui.label("Function preview:");
                ui.label("A  B  |  F");
                ui.label("-----------");

                let fixed_inputs = [
                    (0, 0),
                    (0, 1),
                    (1, 0),
                    (1, 1),
                ];

                if preview.outputs.len() == 4 {
                    for (i, (a, b)) in fixed_inputs.iter().enumerate() {
                        let f = if preview.outputs[i] { 1 } else { 0 };

                        ui.label(format!("{}  {}  |  {}", a, b, f));
                    }
                } else {
                    for (a, b) in fixed_inputs {
                        ui.label(format!("{}  {}  |  ?", a, b));
                    }
                }
            });

            egui::TopBottomPanel::top("Header").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    if ui.button("Back to Menu").clicked() { // Go back to main menu
                        next_state.set(GameState::MainMenu);
                    }

                    // Go to next level
                    if ui.button("To the Next Level").clicked() {
                        let idx = active_circuit.active;

                        for entity in &all_gates {
                            commands.entity(*entity).despawn();
                        }
                        for entity in &all_wires {
                            commands.entity(*entity).despawn();
                        }

                        active_circuit.circuits[idx] = crate::circuit::Circuit::new(2, 1);
                    }
                });
            });
            // Create new draggable window for user to add gates
            egui::Window::new("Components").show(ctx, |ui| {
                // Disable certain buttons depending on level
                if ui // NAND
                    .add_sized([60.0, 30.0], egui::Button::new("NAND"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NAND,
                    });
                }

                if ui //
                    .add_sized([60.0, 30.0], egui::Button::new("NOR"))
                    .clicked()
                {

                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NOR,
                    });
                }

                if ui
                    .add_sized([60.0, 30.0], egui::Button::new("AND"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::AND,
                    });
                }

                if ui
                    .add_sized([60.0, 30.0], egui::Button::new("OR"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::OR,
                    });
                }

                if ui
                    .add_sized([60.0, 30.0], egui::Button::new("XOR"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::XOR,
                    });
                }

                if ui
                    .add_sized([60.0, 30.0], egui::Button::new("XNOR"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::XNOR,
                    });
                }

                if ui
                    .add_sized([60.0, 30.0], egui::Button::new("NOT"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NOT,
                    });
                }
            });
        }

        GameState::Credits => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Credits")
                    .size(100.0)
                    .strong()
                ); // Set label for the window

                ui.separator(); // Add strikethrough border

                // Credits text
                ui.label("Project created by:");
                ui.label("Noah Springer (noahds4)");
                ui.label("Daniel Moraga (dmora59)");
                ui.label("Ferroforge - created for CS128 Honors Project");

                // Go back to main menu if back button pressed
                if ui
                    .add_sized([250.0, 80.0], egui::Button::new("Back to Menu"))
                    .clicked()
                {
                    next_state.set(GameState::MainMenu);
                }
            });
        }
    }

    Ok(())
}

/// Unused function...
pub fn button_system(
    active_circuit: ResMut<ActiveCircuit>,
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(SANDY_BROWN);
                let idx = active_circuit.active;
                if active_circuit.circuits[idx].check_connection() {
                    let input_0 = active_circuit.circuits[idx].evaluate(false, false);
                    let input_1 = active_circuit.circuits[idx].evaluate(false, true);
                    let input_2 = active_circuit.circuits[idx].evaluate(true, false);
                    let input_3 = active_circuit.circuits[idx].evaluate(true, true);
                    let first_output ;
                    let second_output;
                    let third_output;
                    let fourth_output;
                    if input_0 == true {
                        first_output = 1;
                    } else {
                        first_output = 0;
                    }
                    if input_1 == true {
                        second_output = 1;
                    } else {
                        second_output = 0;
                    }
                    if input_2 == true {
                        third_output = 1;
                    } else {
                        third_output = 0;
                    }
                    if input_3 == true {
                        fourth_output = 1;
                    } else {
                        fourth_output = 0;
                    }
                    println!("0 and 0 input leads to {first_output} as the output");
                    println!("0 and 1 input leads to {second_output} as the output");
                    println!("1 and 0 input leads to {third_output} as the output");
                    println!("1 and 1 input leads to {fourth_output} as the output");
                } else {
                    println!("the inputs and outputs aren't correctly connected!");
                }


                // The accessibility system's only update the button's state when the `Button` component is marked as changed.
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

// Create a button
pub fn button(asset_server: &AssetServer, x_pos: f32, y_pos: f32, width: u32, height: u32) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                bottom: px(y_pos),
                right: px(x_pos),
                width: px(width),
                height: px(height),
                border: UiRect::all(px(5)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("Button"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}

/// handle_spawn_gate: Given the message passed from the user_interface function, spawn a new gate
/// 
/// # Arguments
/// 'commands' - Write to world
/// 'events' - Read what messages were written by user_interface
/// 'textures' - Access the textures in assets/textures/...
/// 'active_circuit' - Modify the backend to add new gate node to the graph
pub fn handle_spawn_gate(
    mut commands: Commands,
    mut events: MessageReader<SpawnGateEvent>,
    textures: Res<Textures>,
    mut active_circuit: ResMut<ActiveCircuit>,
) {
    for event in events.read() {

        // Add gate to backend
        let idx = active_circuit.active;
        active_circuit.circuits[idx].add_gate(event.gate_type.clone());

        // Get ID of newly created gate
        let gate_id = active_circuit.circuits[idx].gates.last().unwrap().id;

        // Get the type of gate to set the texture to
        let texture = match event.gate_type.clone() {
            GateType::NAND => textures.nand_gate.clone(),
            GateType::AND => textures.and_gate.clone(),
            GateType::OR => textures.or_gate.clone(),
            GateType::NOR => textures.nor_gate.clone(),
            GateType::NOT => textures.not_gate.clone(),
            GateType::XOR => textures.xor_gate.clone(),
            GateType::XNOR => textures.xnor_gate.clone(),
        };

        // Spawn visual using BlockBundle
        // If not gate, do different spawn since number of children is one, not two
        if event.gate_type.clone() == GateType::NOT {
            commands.spawn(BlockBundle::new(
                event.position,
                texture,
                gate_id,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, 0.0, 10.0), // make on top of everything
                    Port {
                        is_output: false,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(48.0, 0.0, 10.0),
                    Port {
                        is_output: true,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
            });
        } else { // This is not a NOT, spawn two input ports
            commands.spawn(BlockBundle::new(
                event.position,
                texture,
                gate_id,
            ))
            .with_children(|parent| {
            // First port
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, 16.0, 10.0),
                    Port {
                        is_output: false,
                        port_id: 0,
                        identifier: gate_id,
                    }
                ));
                // Second port
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, -16.0, 10.0),
                    Port {
                        is_output: false,
                        port_id: 1,
                        identifier: gate_id,
                    },
                ));
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(48.0, 0.0, 10.0),
                    Port {
                        is_output: true,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
            });
        }
    }
}

/// select_input_port: Update selected input whenever the user left clicks on a port
/// 
/// # Arguments
/// 'state' - Set the connection state for the given entity
/// 'commands' - Modify world by despawning entities
/// 'mouse' - Read the mouse input
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all entities with Transform, GateId components, and are Draggable
pub fn select_input_port(
    mut state: ResMut<ConnectionState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &GlobalTransform, &Port)>,
) {
   
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    
    if state.selected_input.is_some() { // Only run if nothing is selected
        return;
    }

    let Some(cursor) = cursor_to_world(&windows, &cameras) else { return };

    for (entity, transform, port) in &query {
        let dist = transform.translation().truncate().distance(cursor);
        if dist < 10.0 && !port.is_output {
            state.selected_input = Some(entity);
            state.just_selected = true;
            println!("Selected input port: {:?}", entity);
            break;
            
        }
    }
}

/// select_input_port: Update selected input whenever the user left clicks on a port
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'state' - Set the connection state for the given entity
/// 'mouse' - Read the mouse input
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all entities with Transform, GateId components, and are Draggable
/// 'active_circuit' - Represents the backend for deleting the node associated with that gate
pub fn connect_to_output(
    mut commands: Commands,
    mut state: ResMut<ConnectionState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &GlobalTransform, &Port)>,
    wires: Query<&Wire>,
    mut active_circuit: ResMut<ActiveCircuit>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if state.selected_input.is_none() {
        return;
    }

    if state.just_selected {
        state.just_selected = false;
        return; // skip same-frame click
    }

    let Some(input_entity) = state.selected_input else { return };

    // BLOCK: if input already has a wire, reject new connection
    for wire in &wires {
        if wire.to == input_entity {
            println!("Input already occupied");
            state.selected_input = None;
            return;
        }
    }

    let Some(cursor) = cursor_to_world(&windows, &cameras) else { return };

    for (entity, transform, port) in &query {
        // Do not enable clicking on same port
        if entity == input_entity {
            continue;
        }
        
        // check for self-looping, so you can't connect a gate's input to its output
        let dist = transform.translation().truncate().distance(cursor);
        let Ok((_, _, input_port)) = query.get(input_entity) else {
            state.selected_input = None;
            return;
        };
        if port.identifier == input_port.identifier {
            println!("needs to be different gate");
        }

        // Prevent connecting an input to an input
        if dist < 10.0 && port.is_output && port.identifier != input_port.identifier {
            // VALID CONNECTION
            println!("Valid connection found!");

            // Logic behind connecting two gates together, in the current level.
            let input_id = input_port.identifier as usize;
            let output_id = port.identifier as usize;
            let idx = active_circuit.active;
            active_circuit.circuits[idx].connect_gates(input_id, output_id);
            println!("/////////");
            println!("current graph:");
            println!("{:?}", active_circuit.circuits[idx].graph);
            println!("/////////");
            commands.spawn((
                Wire {
                    from: entity,
                    to: input_entity,
                },
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                Transform::default(),
                GlobalTransform::default(),
            ));
            if active_circuit.circuits[idx].check_connection() {
                println!("the gates are connected!");
            }
            println!("Connected {:?} -> {:?}", entity, input_entity);

            break;
        }
    }

    // Always reset after second click attempt
    state.selected_input = None;
}

// Update the wire texture every frame in case the gate is moved
pub fn update_wires(
    mut query: Query<(&Wire, &mut Transform)>,
    port_query: Query<&GlobalTransform, With<Port>>,
) {
    for (wire, mut transform) in &mut query {
        let Ok(from_tf) = port_query.get(wire.from) else { continue };
        let Ok(to_tf) = port_query.get(wire.to) else { continue };

        let start = from_tf.translation();
        let end = to_tf.translation();

        let diff = end - start;
        let length = diff.length();

        transform.translation = (start + end) / 2.0;
        transform.scale = Vec3::new(length, 2.0, 1.0);

        let angle = diff.y.atan2(diff.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

/// select_input_port: Update selected input whenever the user left clicks on a port
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'pos' - Store the x, y, and z position we want to create the board port in
/// 'textures' - Access the textures in assets/textures/...
/// 'is_output' - Determine what type of port to spawn
/// 'identifier' - Determine what texture to use depending on identifier
/// 'port_id' - Create port ID for this port, an unreachable number
pub fn spawn_board_port(
    commands: &mut Commands,
    pos: Vec3,
    textures: &Textures,
    is_output: bool,
    identifier: i32,
    port_id: i32,
) {
    let snapped = Vec3::new(
        snap_to_grid(pos.x),
        snap_to_grid(pos.y),
        pos.z + 10.0, // Make front
    );


    commands.spawn((
        Sprite {
            image: textures.port.clone(),
            color: Color::srgb(0.4, 0.4, 0.4), // grey
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_translation(snapped),
        Port {
            port_id,
            is_output,
            identifier,
        },
    ));
    // Spawn another object with the texture of the respective static thing
    commands.spawn((
        Sprite {
            image: textures.port.clone(),
            color: Color::srgb(0.4, 0.4, 0.4), // grey
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_translation(snapped),
        Port {
            port_id,
            is_output,
            identifier,
        },
    ));

    // Hard code the sprite next to the port
    if is_output { // If true, must be A or B
        if identifier == 10000 { // Must be A
            // decorative label / icon next to it
            commands.spawn((
                Sprite {
                    image: textures.output_a.clone(),
                    custom_size: Some(Vec2::splat(64.0)),
                    ..default()
                },
                Transform::from_translation(snapped + Vec3::new(-32.0, 0.0, 0.1)),
            ));
        } else { // Must be B
            commands.spawn((
                Sprite {
                    image: textures.output_b.clone(),
                    custom_size: Some(Vec2::splat(64.0)),
                    ..default()
                },
                Transform::from_translation(snapped + Vec3::new(-32.0, 0.0, 0.1)),
            ));
        }
    } else { // Must be F
        println!("Spawning F");
        commands.spawn((
            Sprite {
                image: textures.input_f.clone(),
                custom_size: Some(Vec2::splat(64.0)),
                ..default()
            },
           Transform::from_translation(snapped + Vec3::new(32.0, 0.0, 0.1)),
        ));
    }
}

/// cleanup_wires - Every frame, look through every wire and delete it if either to or from are missing
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'wires' - Queue through every single wire in this frame
/// 'ports' - Queue thorugh every single port in this frame
pub fn cleanup_wires(
    mut commands: Commands,
    wires: Query<(Entity, &Wire)>,
    ports: Query<Entity, With<Port>>,
) {
    let alive_ports: std::collections::HashSet<Entity> =
        ports.iter().collect();

    for (wire_entity, wire) in &wires {
        let from_alive = alive_ports.contains(&wire.from);
        let to_alive = alive_ports.contains(&wire.to);

        if !from_alive || !to_alive {
            commands.entity(wire_entity).despawn();
        }
    }
}

/// lighten_on_hover - Detect when the mouse is hovering over a gate, then light it up until mouse is off
/// # Arguments
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all draggable entities
/// 
/// This reuses a lot of delete_on_right_click code...
pub fn lighten_on_hover(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&Transform, &mut Sprite), With<Draggable>>,
) {
    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    for (transform, mut sprite) in &mut query {
        let dist = transform.translation.truncate().distance(cursor_pos);

        let base = Color::srgb(0.8, 0.8, 0.8);
        let hover = Color::srgb(1.0, 1.0, 1.0);

        sprite.color = if dist < 40.0 { hover } else { base };
    }
}

/// lighten_on_hover - Detect when the mouse is hovering over a gate, then light it up until mouse is off
/// # Arguments
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all draggable entities
/// 
/// This reuses a lot of delete_on_right_click code...
pub fn lighten_port_on_hover(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&GlobalTransform, &mut Sprite), With<Port>>,
) {
    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    for (global_transform, mut sprite) in &mut query {
        let world_pos = global_transform.translation().truncate();
        let dist = world_pos.distance(cursor_pos);

        let base = Color::srgb(0.4, 0.4, 0.4);
        let hover = Color::srgb(0.7, 0.7, 0.7);

        sprite.color = if dist < 12.0 { hover } else { base };
    }
}

/// update_circuit_preview - Create circuit preview based on current gate state
/// # Arguments
/// 'active_circuit' - Access the node graph
/// 'preview' - Store a vector of outputs to use in a later function
pub fn update_circuit_preview(
    active_circuit: Res<ActiveCircuit>,
    mut preview: ResMut<CircuitPreview>,
) {
    let circuit = match active_circuit.circuits.get(active_circuit.active) {
        Some(c) => c,
        None => return,
    };

    preview.outputs.clear();

    if !circuit.check_connection() {
        return;
    }

    let test_inputs = [
        (false, false),
        (false, true),
        (true, false),
        (true, true),
    ];

    for (a, b) in test_inputs {
        let result = circuit.evaluate(a, b);
        preview.outputs.push(result);
    }
}