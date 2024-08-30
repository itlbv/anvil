use crate::btree::BehaviorTreeNode;
use crate::components::{Hunger, Movement, Position, Shape};
use crate::window::Window;
use crate::{behaviors, EntityCommand, Knowledge, Properties};
use hecs::{Entity, World};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn choose_behaviors(
    behaviors: &mut HashMap<Entity, Box<dyn BehaviorTreeNode>>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    entity_commands: &mut Vec<EntityCommand>,
    world: &mut World,
) {
    // react to hunger, choose behavior
    for (entity, (hunger)) in world.query_mut::<(&Hunger)>() {
        let mut behavior: Box<dyn BehaviorTreeNode> = behaviors::do_nothing();
        if hunger.value > 3 {
            behavior = behaviors::find_food();
            println!("Behavior updated! Hungry!")
        }
        behaviors.insert(entity, behavior);
    }
}

pub fn run_behaviors(
    behaviors: &mut HashMap<Entity, Box<dyn BehaviorTreeNode>>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    entity_commands: &mut Vec<EntityCommand>,
    world: &mut World,
) {
    behaviors.iter_mut().for_each(|(entity, behavior)| {
        let knowledge = knowledges.get_mut(entity).unwrap();
        behavior.run(knowledge, entity_commands, world);
    });
}

pub fn movement(world: &mut World) {
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
    }
}

pub fn hunger(instant: Instant, world: &mut World) {
    for (_, hunger) in world.query_mut::<&mut Hunger>() {
        if instant - hunger.last_updated > Duration::from_secs(1) {
            hunger.value += 1;
            hunger.last_updated = Instant::now();
        }
    }
}

pub fn draw(window: &mut Window, properties: &Properties, world: &mut World) {
    window.start_frame();

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
}
