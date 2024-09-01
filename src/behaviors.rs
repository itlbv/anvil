use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::btree::{BehaviorStatus, BehaviorTreeNode, Sequence};
use crate::components::StateType::{IDLE, MOVE};
use crate::components::{Food, Movement, Position, State};
use crate::{EntityCommand, EntityCommandType, Knowledge, MoveTask};
use hecs::World as ComponentRegistry;

pub fn do_nothing() -> Box<dyn BehaviorTreeNode> {
    Box::new(DoNothing {})
}

pub fn find_food() -> Box<Sequence> {
    Box::new(Sequence::of(vec![
        FindNearestFood::new(),
        MoveToPickUp::new(),
        // pick up
        // consume
    ]))
}

pub fn move_to() -> Box<dyn BehaviorTreeNode> {
    Box::new(MoveToPickUp {})
}

struct DoNothing {}

impl BehaviorTreeNode for DoNothing {
    fn run(
        &mut self,
        _knowledge: &mut Knowledge,
        _entity_commands: &mut Vec<EntityCommand>,
        _registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        Running
    }
}

struct FindNearestFood {}

impl FindNearestFood {
    fn new() -> Box<Self> {
        Box::new(FindNearestFood {})
    }
}

impl BehaviorTreeNode for FindNearestFood {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        // find own position
        let own_pos = registry.get::<&Position>(knowledge.own_id).unwrap();
        let own_pos_x = own_pos.x;
        let own_pos_y = own_pos.y;
        drop(own_pos);

        // find nearest food
        let mut nearest_food = None;
        let mut smallest_distance = f32::MAX;
        for (food_entity, (_food, pos)) in registry.query_mut::<(&Food, &Position)>() {
            let dist_x = (pos.x - own_pos_x).abs();
            let dist_y = (pos.y - own_pos_y).abs();
            let dist = dist_x.hypot(dist_y);
            if dist < smallest_distance {
                smallest_distance = dist;
                nearest_food = Option::from(food_entity);
            }
        }

        // set target
        match nearest_food {
            None => Failure,
            Some(target_entity) => {
                knowledge.target = Option::from(target_entity);
                println!("Finished finding food");
                Success
            }
        }
    }
}

struct MoveToPickUp {}

impl MoveToPickUp {
    fn new() -> Box<Self> {
        Box::new(MoveToPickUp {})
    }
}

impl BehaviorTreeNode for MoveToPickUp {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        let own_pos = registry.get::<&Position>(knowledge.own_id).unwrap();
        let mut movement = registry.get::<&mut Movement>(knowledge.own_id).unwrap();
        let mut state = registry.get::<&mut State>(knowledge.own_id).unwrap();

        // check if already arrived
        if (own_pos.x - knowledge.destination_x).abs() < movement.distance
            && (own_pos.y - knowledge.destination_y).abs() < movement.distance
        {
            state.state = IDLE;
            println!("Finished movement");
            return Success;
        }

        // start movement
        state.state = MOVE;
        movement.destination_x = knowledge.destination_x;
        movement.destination_y = knowledge.destination_y;
        movement.distance = 0.5;

        Running

        // // issue move command and wait for signal
        // entity_commands.push(EntityCommand {
        //     entity: knowledge.own_id,
        //     event_type: EntityCommandType::ApproachTarget,
        //     param: [
        //         ("x".to_string(), knowledge.destination_x.to_string()),
        //         ("y".to_string(), knowledge.destination_y.to_string()),
        //         ("distance".to_string(), "0.5".to_string()),
        //     ]
        //     .into(),
        // });
        // return Running;
    }
}
