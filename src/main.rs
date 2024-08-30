mod behaviors;
mod btree;
mod components;
mod input_controller;
mod systems;
mod util;
mod window;

use crate::btree::BehaviorTreeNode;
use crate::components::{Food, Hunger, Movement, Position, Shape};
use crate::input_controller::InputController;
use crate::systems::{choose_behaviors, draw, hunger, movement, run_behaviors};
use crate::window::Window;
use crate::EntityCommandType::MoveToPosition;
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
    MoveToPosition,
    ApproachTarget,
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
                MoveToPosition => {
                    let mut move_task = world
                        .get::<&mut Movement>(entity_event.entity)
                        .expect("Error getting Move component");
                    move_task.active = true;
                    move_task.distance = 0.05;
                    move_task.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    move_task.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
                EntityCommandType::ApproachTarget => {
                    let mut move_task = world
                        .get::<&mut Movement>(entity_event.entity)
                        .expect("Error getting Move component");
                    move_task.active = true;
                    move_task.distance = entity_event.param["distance"].parse::<f32>().unwrap();
                    move_task.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    move_task.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
            }
        }

        if instant - behavior_last_updated > Duration::from_secs(10) {
            choose_behaviors(
                &mut behaviors,
                &mut knowledges,
                &mut entity_commands,
                &mut world,
            );
            behavior_last_updated = Instant::now();
        }
        run_behaviors(
            &mut behaviors,
            &mut knowledges,
            &mut entity_commands,
            &mut world,
        );
        movement(&mut world);
        hunger(instant, &mut world);

        draw(&mut window, &properties, &mut world);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        instant = Instant::now();
    }

    Ok(())
}
