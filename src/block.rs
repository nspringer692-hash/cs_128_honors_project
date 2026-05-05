use bevy::prelude::*;
use crate::snap_to_grid;

// Needs to access Draggable
use crate::components::*;


#[derive(Component, Default)]
pub struct GateId(pub i32);

// the BlockBundle struct is essentially the visual aspect of the gates when placed down
// each one gets an id that matches to a value inside the connected gate logic
#[derive(Bundle, Default)]
pub struct BlockBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub draggable: Draggable,
    pub curr_id: GateId,
    //visability
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}


impl BlockBundle {
    pub fn new(pos: Vec3, texture: Handle<Image>, curr_id: i32) -> Self {
        let snapped = Vec3::new(
            snap_to_grid(pos.x),
            snap_to_grid(pos.y),
            pos.z,
        );

        Self {
            sprite: Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(100.0)),
                ..default()
            },
            transform: Transform::from_translation(snapped),
            draggable: Draggable,
            // so that it can have the same id as its gate so it can be extracted
            // if it needs to be deleted
            curr_id: GateId(curr_id),
            ..default()
        }
    }
}