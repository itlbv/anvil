use crate::btree::BehaviorStatus::Running;
use crate::btree::{BehaviorStatus, BehaviorTreeNode};
use crate::components::StateType::Move;
use crate::components::{Hunger, Movement, Position, Shape, State};
use crate::entity_commands::EntityCommand;
use crate::map::Map;
use crate::window::Window;
use crate::{behaviors, BehaviorList, Knowledge, Properties};
use hecs::Entity;
use hecs::World as ComponentRegistry;
use std::collections::HashMap;

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
    let mut keys: Vec<Entity> = behaviors.keys().cloned().collect();
    keys.sort_unstable_by_key(|e| e.to_bits().get());

    for e in keys {
        let bhvs = behaviors.get_mut(&e).expect("behaviours missing entity");
        let knldg = knowledges.get_mut(&e).expect("knowledges missing entity");

        // if behaviors is empty, pring message and assign do_nothing()
        if bhvs.is_empty() {
            println!("All behaviors completed, assigning DoNothing");
            bhvs.push(behaviors::do_nothing())
        }
        // when returned status is not running, remove finished behavior
        let status = bhvs[0].run(knldg, entity_commands, registry);
        match status {
            BehaviorStatus::Success => {
                bhvs.remove(0);
            }
            BehaviorStatus::Failure => {
                bhvs.remove(0);
            }
            _ => {}
        }
    }
}

pub fn movement(registry: &mut ComponentRegistry) {
    for (_, (pos, movement, state)) in
        registry.query_mut::<(&mut Position, &mut Movement, &State)>()
    {
        if state.state != Move {
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

pub fn hunger(dt_seconds: f32, registry: &mut ComponentRegistry) {
    for (_, hunger) in registry.query_mut::<&mut Hunger>() {
        hunger.acc_seconds += dt_seconds;

        // Increment once per full second elapsed; keep remainder in the accumulator.
        while hunger.acc_seconds >= 1.0 {
            hunger.value = hunger.value.saturating_add(1);
            hunger.acc_seconds -= 1.0;
        }
    }
}

pub fn render_frame(
    window: &mut Window,
    properties: &Properties,
    map: &Map,
    registry: &mut ComponentRegistry,
) {
    window.start_frame();

    render_map(window, properties, map);
    render_entites(window, properties, registry);

    window.present_frame();
}

fn render_map(window: &mut Window, properties: &Properties, map: &Map) {
    for map_y in 0..map.height {
        for map_x in 0..map.width {
            window.draw_rect(map_x as f32, map_y as f32, 1., 1., (100, 100, 100, 100))
        }
    }

    if properties.draw_map_grid {
        for x in 0..=map.width {
            // vertical lines
            window.draw_line(
                (x as f32, 0.),
                (x as f32, map.height as f32),
                (0, 0, 0, 255),
            );
        }

        for y in 0..=map.height {
            // horizontal lines
            window.draw_line((0., y as f32), (map.width as f32, y as f32), (0, 0, 0, 255));
        }
    }
}

fn render_entites(window: &mut Window, properties: &Properties, registry: &mut ComponentRegistry) {
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
}
