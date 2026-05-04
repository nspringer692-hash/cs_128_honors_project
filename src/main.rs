use bevy::color::palettes::css::SANDY_BROWN;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};

pub mod gate;
pub mod circuit;
pub mod block;


use gate::{Gate, GateType};
use circuit::Circuit;
use crate::circuit::ActiveCircuit;
use crate::block::{BlockBundle, GateId};
use crate::gate::GLOBAL_ID;

// Components are instance variables per entity in the world

#[derive(Resource)]
struct CurrentStat {
    input: bool,
    working_output: bool,
    output: i32,
}

#[derive(Resource)]
struct GateTexture {
    texture: Handle<Image>,
}

// All components used for dragging stuff
#[derive(Component, Default)]
struct Draggable;

#[derive(Resource, Default)]
struct DragState {
    entity: Option<Entity>,
}


// Each gate will have inputs and outputs
#[derive(Component)]
struct Inputs {
    in_a: bool,
    in_b: bool,
}

#[derive(Component)]
struct Output {
    out: bool,
}

// Events are instances in Bevy that do something in that event (word bad)
#[derive(Message)]
struct SpawnGateEvent {
    position: Vec3,
    gate_type: GateType,
}

// List of game states to track for UI transitions
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    Editor,
    Credits,
}

// Used for snapping to grid
const GRID_SIZE: f32 = 16.0;

// Overall startup, creating the app, running throught the assets and running the program.
// Gives all the initial values that may be needed, for example the current level will be staged on setup and once the program is run
fn main() {
    App::new() // Create new app
    .insert_resource(ActiveCircuit(crate::circuit::Circuit::new(0, 0)))
    .insert_resource(DragState::default()) // Create new global resource to track drag state
    .add_plugins(DefaultPlugins) // Plugins for Bevy game development
    .add_plugins(EguiPlugin::default()) // Plugins for Bevy egui
    .insert_resource(CurrentStat {
        input: false,
        working_output: false,
        output: -1,
    })

    .init_state::<GameState>() // Set initial game state
    .init_resource::<InputFocus>()
    .add_systems(Startup, setup) // Run setup process once
    .add_systems(Update, button_system)
    .add_systems(EguiPrimaryContextPass, user_interface) // Load user interface
    .add_systems(Update, ( // Run certain functions once per frame / every 60 secs
        start_drag_system,
        drag_system,
        end_drag_system,
        delete_on_right_click,
        process_circuit_simulation,
    ))
    .add_message::<SpawnGateEvent>()
    .run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);


// Run the UI system for rendering egui menus + state logic
// Run every frame and depending on whenever a state button is pressed, render different UI
fn user_interface(
    mut contexts: EguiContexts, // Give access to egui to draw UI
    mut active_circuit: ResMut<ActiveCircuit>, // This is the current circuit (or level)
    state: Res<State<GameState>>, // Read what state the game is currently in
    mut next_state: ResMut<NextState<GameState>>, // What state to change to next frame?
    mut message_writer: MessageWriter<SpawnGateEvent>,
    mut commands: Commands, 
    gate_texture: Res<GateTexture>,
) -> Result {
    let ctx = contexts.ctx_mut()?; // Get access to bevy_egui's internal state
    let current_level = &mut active_circuit.0;
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
            egui::Window::new("Components").show(ctx, |ui| {
                // here, the pos is the location that the gate will spawn in and global
                // is the current id that the spawned block will be
                let pos = Vec3::new(-80.0, 0.0, 0.0);
                let global = {
                    let guard = GLOBAL_ID.lock().unwrap();
                    *guard // The lock is released right after this closing brace
                };
                if ui // NAND
                    .add_sized([60.0, 30.0], egui::Button::new("NAND"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NAND,
                    });
                    //spawning in the block and adding the gate to the circuit, continued for all gate types
                    commands.spawn(BlockBundle::new(pos, gate_texture.texture.clone(), global));
                    current_level.add_gate(GateType::NAND);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                    println!("/////////");
                    println!("current graph:");
                    println!("{:?}", current_level.graph);
                    println!("/////////");
                    
                }

                if ui // NOR
                    .add_sized([60.0, 30.0], egui::Button::new("NOR"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NOR,
                    });
                    commands.spawn(BlockBundle::new(pos, gate_texture.texture.clone(), global));
                    current_level.add_gate(GateType::NOR);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                    println!("/////////");
                    println!("current graph:");
                    println!("{:?}", current_level.graph);
                    println!("/////////");
                }

                if ui // AND
                    .add_sized([60.0, 30.0], egui::Button::new("AND"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::AND,
                    });
                    commands.spawn(BlockBundle::new(pos, gate_texture.texture.clone(), global));
                    current_level.add_gate(GateType::AND);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                    println!("/////////");
                    println!("current graph:");
                    println!("{:?}", current_level.graph);
                    println!("/////////");
                }

                if ui // OR
                    .add_sized([60.0, 30.0], egui::Button::new("OR"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::OR,
                    });
                    commands.spawn(BlockBundle::new(pos, gate_texture.texture.clone(), global));
                    current_level.add_gate(GateType::OR);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                    println!("/////////");
                    println!("current graph:");
                    println!("{:?}", current_level.graph);
                    println!("/////////");

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

                // Button logic
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
fn button_system(
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
fn button(asset_server: &AssetServer, x_pos: f32, y_pos: f32, width: u32, height: u32) -> impl Bundle {
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

// fn handle_spawn_gate(
//     mut commands: Commands,
//     mut events: MessageReader<SpawnGateEvent>,
//     gate_texture: Res<GateTexture>
// ) {
//     for event in events.read() {
//         match event.gate_type {
            
//             _ => {
//             }
//         }
//     }
// }

// creates the texture of the gates themselves, while using nand.png. Setting these objects
// in the set coords, for example Vec3::new(-100.0, 0.0, 0.0) is put in the set coords given.

//used in setting up the system *
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(button(&asset_server, 450.0, 320.0, 125, 60));
    let gate_texture: Handle<Image> = asset_server.load("textures/nand.png");
    spawn_grid(&mut commands);

    // Store as a resource
    commands.insert_resource(GateTexture {
        texture: gate_texture.clone(),
    });

}


// Visual grid for workspace
fn spawn_grid(commands: &mut Commands) {
    let spacing = 16.0;
    let half_size = 2000.0;

    let mut x = -half_size;
    while x <= half_size {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.2, 0.2, 0.2, 0.3),
                custom_size: Some(Vec2::new(1.0, half_size * 2.0)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, -10.0),
        ));
        x += spacing;
    }

    let mut y = -half_size;
    while y <= half_size {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.2, 0.2, 0.2, 0.3),
                custom_size: Some(Vec2::new(half_size * 2.0, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, -10.0),
        ));
        y += spacing;
    }
}


// Delete a block whenever hovering and right click is pressed
// This should also maintain gates and keep track of the amount and which gates are "active"
fn delete_on_right_click(
    mut commands: Commands, // Needed to run despawn entity
    mouse: Res<ButtonInput<MouseButton>>, // Read mouse's input
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
    // Loop through each entity in the world
    // This loop goes through until it finds an entity < 50 in distance and deletes it, only one, though
    for (entity, transform, gate_id) in &query {
        // Get the distance from the entity
        
        let dist = transform.translation.truncate().distance(cursor_pos);

        // If this entity is the closest to the mouse, delete it
        if dist < 50.0 {
            // Once the entity that has been found, the process of deletion is started, taking a gate and deleting it.
            // This is done by getting the id of the gate, and deleting based on that id, after which the visuals for
            // the item are removed.                  |
            //                                        v .0 is just the value given (inside the 1-value tuple)
            active_circuit.0.remove_gate(gate_id.0);
            println!("{:?}", active_circuit.0.graph);
            commands.entity(entity).despawn();
            // for i in 0..active_circuit.0.gates.len() {
            //     println!("{:?}", active_circuit.0.gates[i]);
            // }
            break; // delete only one
        }
    }
}

fn process_circuit_simulation(mut active_circuit: ResMut<ActiveCircuit>) {
    let circuit = &mut active_circuit.0;
}

// Work
// Convert cursor to world position
fn cursor_to_world(
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
fn start_drag_system(
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
fn drag_system(
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
fn snap_to_grid(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

// Stop dragging
fn end_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.entity = None;
    }
}