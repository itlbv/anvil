mod components;
mod input_controller;
mod util;
mod window;

use crate::components::{Hunger, Movement, Position, Shape};
use crate::input_controller::InputController;
use crate::window::Window;
use crate::EntityEventType::Move;
use hecs::{Entity, World};
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
}

#[derive(PartialEq)]
enum EntityEventType {
    Move,
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

    let mut properties = Properties {
        quit: false,
        selected_entity: None,
    };

    let mut world = World::new();
    world.spawn((
        Position::new(1., 1.),
        Shape::new(0.4, 0.4, (150, 150, 150, 150)),
        Hunger::new(),
        Movement::new(),
    ));

    let mut entity_events: Vec<EntityEvent> = vec![];

    let mut instant = Instant::now();
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
                Move => {
                    let mut move_task = world
                        .get::<&mut Movement>(entity_event.entity)
                        .expect("Error getting Move component");
                    move_task.active = true;
                    move_task.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    move_task.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
            }
        }

        // hunger
        for (_, hunger) in world.query_mut::<&mut Hunger>() {
            if hunger.last_updated - instant > Duration::from_secs(10) {
                hunger.value += 1;
            }
        }

        // move
        for (_, (pos, movement)) in world.query_mut::<(&mut Position, &mut Movement)>() {
            if !movement.active {
                continue;
            }

            // get distance to destination
            let dist_x = movement.destination_x - pos.x;
            let dist_y = movement.destination_y - pos.y;

            // normalise direction
            let direction_x = dist_x / dist_x.hypot(dist_y);
            let direction_y = dist_y / dist_x.hypot(dist_y);

            // modify position
            pos.x += direction_x * 0.07;
            pos.y += direction_y * 0.07;

            // movement is done
            if movement.destination_x - pos.x < 0.05 && movement.destination_y - pos.y < 0.05 {
                pos.x = movement.destination_x;
                pos.y = movement.destination_y;
                movement.active = false;
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
        instant = Instant::now();
    }

    Ok(())
}
