mod behaviors;
mod btree;
mod components;
mod input_controller;
mod util;
mod window;

use crate::btree::BehaviorTreeNode;
use crate::components::{Food, Hunger, Movement, Position, Shape};
use crate::input_controller::InputController;
use crate::window::Window;
use crate::EntityCommandType::Move;
use hecs::{Entity, World};
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
}

#[derive(PartialEq)]
enum EntityCommandType {
    Move,
}

struct EntityCommand {
    entity: Entity,
    event_type: EntityCommandType,
    param: HashMap<String, String>,
}

struct Knowledge {
    id: Entity,
    target: Option<Entity>,
    map: HashMap<String, String>,
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

    let mut rand = rand::thread_rng();
    let food_to_spawn = (0..6).map(|_| {
        let pos = Position::new(rand.gen_range(2..10) as f32, rand.gen_range(2..10) as f32);
        let shape = Shape::new(0.2, 0.2, (150, 40, 40, 255));
        let food = Food {};
        (pos, shape, food)
    });
    world.spawn_batch(food_to_spawn);

    let entity = world.spawn((
        Position::new(1., 1.),
        Shape::new(0.4, 0.4, (150, 150, 150, 150)),
        Hunger::new(),
        Movement::new(),
    ));

    let mut entity_commands: Vec<EntityCommand> = vec![];
    let mut behaviors: HashMap<Entity, Box<dyn BehaviorTreeNode>> = HashMap::new();
    behaviors.insert(entity, behaviors::do_nothing());

    let mut knowledges: HashMap<Entity, Knowledge> = HashMap::new();
    knowledges.insert(
        entity,
        Knowledge {
            id: entity,
            target: None,
            map: Default::default(),
        },
    );

    let mut instant = Instant::now();
    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut entity_commands, &mut world);

        // process entity commands
        while !entity_commands.is_empty() {
            let entity_event = entity_commands.pop().unwrap();

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

        // choose behaviors
        for (id, (hunger)) in world.query_mut::<(&Hunger)>() {
            let mut behavior: Box<dyn BehaviorTreeNode> = behaviors::do_nothing();
            if hunger.value > 3 {
                behavior = behaviors::find_food();
            }
            behaviors.insert(id, behavior);
        }

        // run behaviors
        behaviors.iter_mut().for_each(|(entity, behavior)| {
            let knowledge = knowledges.get_mut(entity).unwrap();
            behavior.run(knowledge, &mut world);
        });

        // hunger
        for (_, hunger) in world.query_mut::<&mut Hunger>() {
            if instant - hunger.last_updated > Duration::from_secs(1) {
                hunger.value += 1;
                hunger.last_updated = Instant::now();
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
