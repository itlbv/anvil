use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::btree::{BehaviorStatus, BehaviorTreeNode, Sequence};
use crate::components::{Food, Position};
use crate::{EntityCommand, EntityCommandType, Knowledge};
use hecs::World;

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
        _world: &mut World,
    ) -> BehaviorStatus {
        Success
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
        world: &mut World,
    ) -> BehaviorStatus {
        // find own position
        let own_pos = world.get::<&Position>(knowledge.id).unwrap();
        let own_pos_x = own_pos.x;
        let own_pos_y = own_pos.y;
        drop(own_pos);

        // find nearest food
        let mut nearest_food = None;
        let mut smallest_distance = f32::MAX;
        for (id, (_food, pos)) in world.query_mut::<(&Food, &Position)>() {
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
        world: &mut World,
    ) -> BehaviorStatus {
        // if no target, print and exit with failure
        if knowledge.target.is_none() {
            println!("Can't execute MoveToPickUp: no target set.");
            return Failure;
        }

        // issue move command and wait for signal
        let target = knowledge.target.unwrap();
        let target_pos = world.get::<&Position>(target).unwrap();
        let target_x = target_pos.x;
        let target_y = target_pos.y;
        entity_commands.push(EntityCommand {
            entity: knowledge.id,
            event_type: EntityCommandType::ApproachTarget,
            param: [
                ("x".to_string(), target_x.to_string()),
                ("y".to_string(), target_y.to_string()),
                ("distance".to_string(), "0.5".to_string()),
            ]
            .into(),
        });
        return Running;
    }
}
