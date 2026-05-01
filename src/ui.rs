use bevy::color::palettes::css::SANDY_BROWN;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::{input_focus::InputFocus};

use crate::gate::*;
use crate::textures::*;
use crate::components::*;

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

// Visual grid for workspace
pub fn spawn_grid(commands: &mut Commands) {
    let spacing = 16.0;
    let half_size = 2000.0;

    let mut x = -half_size;
    while x <= half_size {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.4, 0.4, 0.3),
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
                color: Color::srgba(0.4, 0.4, 0.4, 0.3),
                custom_size: Some(Vec2::new(half_size * 2.0, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, -10.0),
        ));
        y += spacing;
    }
}


//Helper function, creates said object, a movable gate, usually.
pub fn spawn_block(commands: &mut Commands, pos: Vec3, textures: &Textures) {

    // Snap the position of this object to the grid
    let snapped = Vec3::new(
        snap_to_grid(pos.x),
        snap_to_grid(pos.y),
        pos.z, // pos.z is irrelevant since it's a 2D game
    );
    commands.spawn((
        Sprite {
            image: textures.gate.clone(),
            custom_size: Some(Vec2::splat(100.0)),
            ..default()
        },
        Transform::from_translation(snapped),
        Draggable,
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
                port_id: 0
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
                port_id: 1
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
                port_id: 0
            },
        ));
    });
}

//spawns a stable block, or a block that can't be moved
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

// Delete a block whenever hovering and right click is pressed
pub fn delete_on_right_click(
    mut commands: Commands, // Needed to run despawn entity
    mouse: Res<ButtonInput<MouseButton>>, // Read mouse's input
    windows: Query<&Window>, 
    cameras: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Transform), With<Draggable>>,
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
    for (entity, transform) in &query {
        // Get the distance from the entity
        let dist = transform.translation.truncate().distance(cursor_pos);

        // If this entity is the closest to the mouse, delete it
        if dist < 50.0 {
            commands.entity(entity).despawn();
            break; // delete only one
        }
    }
}

// Get the cursor's position, used for detecting which object is clicked
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
    mut popup: ResMut<PopupState>
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
                let mut current_level = crate::circuit::Circuit::new(0, 5);
                if ui // NAND
                    .add_sized([60.0, 30.0], egui::Button::new("NAND"))
                    .clicked()
                {
                    message_writer.write(SpawnGateEvent {
                        position: Vec3::new(-80.0, 0.0, 0.0),
                        gate_type: GateType::NAND,
                    });
                    current_level.add_gate(GateType::NAND);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                    
                }

                if ui // NOR (placeholder)
                    .add_sized([60.0, 30.0], egui::Button::new("NOR"))
                    .clicked()
                {

                    println!("Request NOR gate");
                    current_level.add_gate(GateType::NOR);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                }

                if ui // AND (placeholder)
                    .add_sized([60.0, 30.0], egui::Button::new("AND"))
                    .clicked()
                {
                    println!("Request AND gate");
                    current_level.add_gate(GateType::NAND);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }
                }

                if ui // OR (placeholder)
                    .add_sized([60.0, 30.0], egui::Button::new("OR"))
                    .clicked()
                {
                    println!("Request OR gate");
                    current_level.add_gate(GateType::OR);
                    for i in 0..current_level.gates.len() {
                        println!("{:?}", current_level.gates[i]);
                    }

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

pub fn handle_spawn_gate(
    mut commands: Commands,
    mut events: MessageReader<SpawnGateEvent>,
    textures: Res<Textures>
) {
    for event in events.read() {
        match event.gate_type {
            GateType::NAND => {
                spawn_block(&mut commands, event.position, &textures);
            }
            GateType::NOR => {
                spawn_block(&mut commands, event.position, &textures);
            }
            GateType::AND => {
                spawn_block(&mut commands, event.position, &textures);
            }
            GateType::OR => {
                spawn_block(&mut commands, event.position, &textures);
            }
            _ => {
                spawn_block(&mut commands, event.position, &textures);
            }
        }
    }
}
