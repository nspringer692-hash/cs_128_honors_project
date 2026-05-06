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

// Determine whether to show the help popup or not
#[derive(Resource, Default)]
pub struct PopupState {
    show_popup : bool,
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
}

// Used for snapping to grid
const GRID_SIZE: f32 = 16.0;

/* 
    Bevy runs on an ECS system
    This is important to know since function parameters are basically:
    "Give me access to this data when the system runs"
*/

// Visual grid for workspace
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

//spawns a stable block, or a block that can't be moved
// USE LATER :)
pub fn spawn_stable_block(commands: &mut Commands, pos: Vec3, texture: Handle<Image>) {
    commands.spawn((
        Sprite {
            image: texture,
            custom_size: Some(Vec2::splat(100.0)),
            ..default()
        },
        Transform::from_translation(pos),
        // Use a system to handle pickable interactions instead of trying to attach
        // a callback here (the `On::<Pointer<Click>>::run` API is not available).
    ));
}

/// Delete a gate and connected wires when the user right-clicks near it.
/// Only delete one entity per click
/// TODO: Also delete the wires it is connected to

/// https://docs.rs/bevy/latest/bevy/prelude/struct.Transform.html
/// Referenced here to access the distance between an entity and a given point
/// 
/// https://bevy-cheatbook.github.io/input/mouse.html
/// Referenced here to figure out how to access cursor clicks
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all entities with Transform, GateId components, and are Draggable
/// 'wires' - Access all entities with a Wire component
/// 'active_circuit' - Represents the backend for deleting the node associated with that gate
pub fn delete_on_right_click(
    mut commands: Commands, // Needed to run despawn entity
    mouse: Res<ButtonInput<MouseButton>>, // Read mouse's input
    windows: Query<&Window>, // Read over all Windows
    cameras: Query<(&Camera, &GlobalTransform)>, // Access screen coordinates
    query: Query<(Entity, &Transform, &GateId), With<Draggable>>, // Access all entities with Draggable component
    wires: Query<(Entity, &Wire)>, // Access all entities with a Wire component
    mut active_circuit: ResMut<ActiveCircuit>, // Modify the backend
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

        // If this entity is 50 units or less away from the mouse (hovered on), delete
        if dist < 50.0 {

            // Delete connected wires
            // TODO: FIX, CURRENTLY DOES NOT WORK AS INTENDED
            for (wire_entity, wire) in &wires {
                if wire.from == entity || wire.to == entity {
                    commands.entity(wire_entity).despawn();
                    break;
                }
            }
            
            // Delete the gate and its children
            commands.entity(entity).despawn();

            // Remove this gate from the graph
            active_circuit.0.remove_gate(gate_id.0);
            break; // We only need to delete one!
        }
    }
}

/// Helper: cursor_to_world
/// cursor_to_world: Get the cursor's position, used for detecting which object is clicked
/// 
/// # Arguments
/// 'commands' - Modify world by despawning entities
/// 'windows' - Read over all windows to get cursor position on screen
/// 'cameras' - Access screen coordinates
/// 'query' - Access all entities with Transform, GateId components, and are Draggable
/// 'wires' - Access all entities with a Wire component
/// 'active_circuit' - Represents the backend for deleting the node associated with that gate
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

// Start dragging on click
//enables the ability to drag objects given
/// 
pub fn start_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Transform), With<Draggable>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    // naive hit test (good for circles/small objects)
    for (entity, transform) in &query {
        let dist = transform.translation.truncate().distance(cursor_pos);

        if dist < 20.0 {
            drag_state.entity = Some(entity);
            break;
        }
    }
}

// Update dragged entity position
pub fn drag_system(
    drag_state: Res<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform>,
) {
    let Some(entity) = drag_state.entity else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_pos) = cursor_to_world(&windows, &cameras) else {
        return;
    };

    if let Ok(mut transform) = query.get_mut(entity) {
        transform.translation.x = snap_to_grid(cursor_pos.x);
        transform.translation.y = snap_to_grid(cursor_pos.y);
    }
}

// Helper: snap to grid
pub fn snap_to_grid(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

// Stop dragging
pub fn end_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.entity = None;
    }
}

// Run the UI system for rendering egui menus + state logic
// Run every frame and depending on whenever a state button is pressed, render different UI
pub fn user_interface(
    mut contexts: EguiContexts, // Give access to egui to draw UI
    state: Res<State<GameState>>, // Read what state the game is currently in
    mut next_state: ResMut<NextState<GameState>>, // What state to change to next frame?
    mut message_writer: MessageWriter<SpawnGateEvent>,
    mut popup: ResMut<PopupState>,
    _commands: Commands,
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
                    .add_sized([250.0, 80.0], egui::Button::new("Start Editor"))
                    .clicked()
                {
                    next_state.set(GameState::Editor);
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

        GameState::Editor => { // If editor, show editor
            egui::SidePanel::left("Panel").show(ctx, |ui| {
                if ui.button("Back to Menu").clicked() { // Go back to main menu
                    next_state.set(GameState::MainMenu);
                }
                

                ui.label("Editor Mode"); // Set header as Editor Mode
            });

            egui::TopBottomPanel::top("Header").show(ctx, |ui| {
                // The top panel is often a good place for a menu bar:
                egui::MenuBar::new().ui(ui, |ui| {
                    if ui.button("Back to Menu").clicked() { // Go back to main menu
                        next_state.set(GameState::MainMenu);
                    }

                     if ui.button("Help").clicked() { // Go back to main menu
                        popup.show_popup = true;
                    }

                    if popup.show_popup { // No need to account for a close button since bevy egui includes that in windows!
                        egui::Window::new("Help")
                        .vscroll(true)
                        .open(&mut popup.show_popup)
                        .show(ctx, |ui| {
                            ui.label("This is a placeholder");
                        });
                    }
                });
            });
            // Create new draggable window for user to add gates
            // WIP
            egui::Window::new("Components").show(ctx, |ui| {
                // let mut current_level = crate::circuit::Circuit::new(0, 5);
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

//this button will initialize the input value, whether it is true (1) or false (0) and this will help with testing later
//may be changed in the future
// WIP
pub fn button_system(
    mut current_status: ResMut<CurrentStat>,
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
                current_status.input = !current_status.input;
                if current_status.input {
                    println!("1");
                } else {
                    println!("0");
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

//use this function to make a button that can be placed in the x_pos, and set its size
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

// Spawn a gate and add it to the backend when a gate is passed from the gate spawn button
pub fn handle_spawn_gate(
    mut commands: Commands,
    mut events: MessageReader<SpawnGateEvent>,
    textures: Res<Textures>,
    mut active_circuit: ResMut<ActiveCircuit>,
) {
    for event in events.read() {

        // Add gate to backend
        active_circuit.0.add_gate(event.gate_type.clone());
        // println!("/////////");
        // println!("current graph:");
        // println!("{:?}", active_circuit.0.graph);
        // println!("/////////");

        // Get ID of newly created gate
        let gate_id = active_circuit.0.gates.last().unwrap().id;

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
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, 0.0, 1.0),
                    Port {
                        is_output: false,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(48.0, 0.0, 1.0),
                    Port {
                        is_output: true,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
            });
        } else {
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
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, 16.0, 1.0),
                    Port {
                        is_output: false,
                        port_id: 0,
                        identifier: gate_id,
                    }
                ));
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(-48.0, -16.0, 1.0),
                    Port {
                        is_output: false,
                        port_id: 1,
                        identifier: gate_id,
                    },
                ));
                parent.spawn((
                    Sprite {
                        image: textures.port.clone(),
                        color: Color::srgb(1.0, 0.3, 0.3),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(48.0, 0.0, 1.0),
                    Port {
                        is_output: true,
                        port_id: 0,
                        identifier: gate_id,
                    },
                ));
            });
        }

        // match event.gate_type {
        //     GateType::NAND => {
        //         spawn_block(&mut commands, event.position, &textures);
        //     }
        //     GateType::NOR => {
        //         spawn_block(&mut commands, event.position, &textures);
        //     }
        //     GateType::AND => {
        //         spawn_block(&mut commands, event.position, &textures);
        //     }
        //     GateType::OR => {
        //         spawn_block(&mut commands, event.position, &textures);
        //     }
        //     _ => {
        //         spawn_block(&mut commands, event.position, &textures);
        //     }
        // }
    }
}

// Function for detecting the user selecting an input port
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
        // if state.just_selected {
        //     sprite.color = Color::GREEN;
        // } else {
        //     sprite.color = Color::RED;
        // }
    }
}

// Function for detecting the user selecting an output port after input port
pub fn connect_to_output(
    mut commands: Commands,
    mut state: ResMut<ConnectionState>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &GlobalTransform, &Port)>,
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
            active_circuit.0.connect_gates(input_id, output_id);
            println!("/////////");
            println!("current graph:");
            println!("{:?}", active_circuit.0.graph);
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
            if active_circuit.0.check_connection() {
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
        pos.z,
    );


    commands.spawn((
        Sprite {
            image: textures.port.clone(),
            color: Color::srgb(1.0, 0.3, 0.3),
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
}

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