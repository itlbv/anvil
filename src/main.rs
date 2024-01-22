mod input_controller;
mod window;

use crate::input_controller::InputController;
use crate::window::Window;
use crate::EntityEventType::{StartMove, StopMove};
use hecs::{Entity, World};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::{EventPump, Sdl};
use std::collections::HashMap;
use std::time::Duration;

fn world_to_screen(world: f32, zoom_factor: usize) -> i32 {
    (world * zoom_factor as f32) as i32
}

fn screen_to_world(screen: i32, zoom_factor: usize) -> f32 {
    screen as f32 / zoom_factor as f32
}

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
}

struct Position {
    pub x: f32,
    pub y: f32,
}

struct Shape {
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8, u8),
}

struct Move {
    active: bool,
    destination_x: f32,
    destination_y: f32,
}

#[derive(PartialEq)]
enum EntityEventType {
    StartMove,
    StopMove,
}

struct EntityEvent {
    entity: Entity,
    event_type: EntityEventType,
    param: HashMap<String, String>,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut input_controller = InputController::new(&sdl_context);

    let mut world = World::new();
    let mut properties = Properties {
        quit: false,
        selected_entity: None,
    };

    world.spawn((
        Move {
            active: false,
            destination_x: 0.0,
            destination_y: 0.0,
        },
        Position { x: 1., y: 1. },
        Shape {
            width: 0.4,
            height: 0.4,
            color: (150, 150, 150, 255),
        },
    ));

    let mut entity_events: Vec<EntityEvent> = vec![];
    let mut iterations = 0;

    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut entity_events, &mut world);

        // entity_events
        while !entity_events.is_empty() {
            let entity_event = entity_events.pop().unwrap();

            match entity_event.event_type {
                StartMove => {
                    let mut move_task = world
                        .get::<&mut Move>(entity_event.entity)
                        .expect("Error getting Move component");
                    move_task.active = true;
                    move_task.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    move_task.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
                StopMove => {}
            }
        }

        // move
        for (id, (pos, move_task)) in world.query_mut::<(&mut Position, &mut Move)>() {
            if !move_task.active {
                continue;
            }

            // get distance to destination
            let mut dist_x = move_task.destination_x - pos.x;
            let mut dist_y = move_task.destination_y - pos.y;

            // normalise direction
            let direction_x = dist_x / dist_x.hypot(dist_y);
            let direction_y = dist_y / dist_x.hypot(dist_y);

            // modify position
            pos.x += direction_x * 0.07;
            pos.y += direction_y * 0.07;

            // movement is done
            if move_task.destination_x - pos.x < 0.05 && move_task.destination_y - pos.y < 0.05 {
                pos.x = move_task.destination_x;
                pos.y = move_task.destination_y;
                move_task.active = false;
                println!("Movement is done")
            }
        }

        window.start_frame();

        // draw entities
        for (id, (pos, shape)) in world.query_mut::<(&Position, &Shape)>() {
            window.draw_rect(
                pos.x - shape.width / 2.,
                pos.y - shape.width / 2.,
                shape.width,
                shape.height,
                shape.color,
            );
            window.draw_dot(pos.x, pos.y, (255, 255, 255, 255));

            // draw selection marker if entity is selected
            match properties.selected_entity {
                None => {}
                Some(selected_entity) => {
                    if selected_entity == id {
                        window.draw_selection_marker(pos.x, pos.y);
                    }
                }
            }
        }

        window.present_frame();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        iterations += 1;
    }

    Ok(())
}
