use crate::btree::BehaviorStatus::Running;
use crate::btree::{BehaviorStatus, BehaviorTreeNode};
use crate::components::StateType::MOVE;
use crate::components::{Hunger, Movement, Position, Shape, State};
use crate::window::Window;
use crate::{behaviors, BehaviorList, EntityCommand, Knowledge, Properties};
use hecs::Entity;
use hecs::World as ComponentRegistry;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn choose_behaviors(
    behaviors: &mut HashMap<Entity, BehaviorList>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    entity_commands: &mut Vec<EntityCommand>,
    registry: &mut ComponentRegistry,
) {
    // react to hunger, choose behavior
    for (entity, (hunger)) in registry.query_mut::<(&Hunger)>() {
        let mut behavior = behaviors::do_nothing();
        if hunger.value > 3 {
            behavior = behaviors::find_food();
            println!("Behavior updated! Hungry!")
        }
        behaviors.insert(entity, vec![behavior]);
    }
}

pub fn run_behaviors(
    behaviors: &mut HashMap<Entity, BehaviorList>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    entity_commands: &mut Vec<EntityCommand>,
    registry: &mut ComponentRegistry,
) {
    behaviors.iter_mut().for_each(|(entity, behavior_vec)| {
        let knowledge = knowledges.get_mut(entity).unwrap();
        // if behaviors is empty, pring message and assign do_nothing()
        if behavior_vec.is_empty() {
            println!("All behaviors completed, assigning DoNothing");
            behavior_vec.push(behaviors::do_nothing())
        }
        // when returned status is not running, remove finished behavior
        let status = behavior_vec[0].run(knowledge, entity_commands, registry);
        match status {
            BehaviorStatus::Success => {
                behavior_vec.remove(0);
            }
            BehaviorStatus::Failure => {
                behavior_vec.remove(0);
            }
            _ => {}
        }
    });
}

pub fn movement(registry: &mut ComponentRegistry) {
    for (_, (pos, movement, state)) in
        registry.query_mut::<(&mut Position, &mut Movement, &State)>()
    {
        if state.state != MOVE {
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

pub fn hunger(instant: Instant, registry: &mut ComponentRegistry) {
    for (_, hunger) in registry.query_mut::<&mut Hunger>() {
        if instant - hunger.last_updated > Duration::from_secs(1) {
            hunger.value += 1;
            hunger.last_updated = Instant::now();
        }
    }
}

pub fn draw(window: &mut Window, properties: &Properties, registry: &mut ComponentRegistry) {
    window.start_frame();

    for (id, (pos, shape)) in registry.query_mut::<(&Position, &Shape)>() {
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
