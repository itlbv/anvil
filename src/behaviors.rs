use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::btree::{BehaviorStatus, BehaviorTreeNode, Sequence};
use crate::components::StateType::IDLE;
use crate::components::{Food, Movement, Position, State};
use crate::{EntityCommand, EntityCommandType, Knowledge};
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
        for (id, (_food, pos)) in registry.query_mut::<(&Food, &Position)>() {
            let dist_x = (pos.x - own_pos_x).abs();
            let dist_y = (pos.y - own_pos_y).abs();
            let dist = dist_x.hypot(dist_y);
            if dist < smallest_distance {
                smallest_distance = dist;
                nearest_food = Option::from(id);
            }
        }

        // set target
        match nearest_food {
            None => Failure,
            Some(id) => {
                knowledge.target = Option::from(id);
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
        // if no target, print and exit with failure
        if knowledge.target.is_none() {
            println!("Can't execute MoveToPickUp: no target set.");
            return Failure;
        }

        let target = knowledge.target.unwrap();
        let target_pos = registry.get::<&Position>(target).unwrap();
        let own_pos = registry.get::<&Position>(knowledge.own_id).unwrap();
        let mut movement = registry.get::<&mut Movement>(knowledge.own_id).unwrap();
        if (own_pos.x - target_pos.x).abs() < movement.distance
            && (own_pos.y - target_pos.y).abs() < movement.distance
        {
            let mut state = registry.get::<&mut State>(knowledge.own_id).unwrap();
            state.state = IDLE;
            println!("Finished movement");
            return Success;
        }

        // issue move command and wait for signal
        entity_commands.push(EntityCommand {
            entity: knowledge.own_id,
            event_type: EntityCommandType::ApproachTarget,
            param: [
                ("x".to_string(), target_pos.x.to_string()),
                ("y".to_string(), target_pos.y.to_string()),
                ("distance".to_string(), "0.5".to_string()),
            ]
            .into(),
        });
        return Running;
    }
}
