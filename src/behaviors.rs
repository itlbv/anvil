use crate::btree::BehaviorStatus::Success;
use crate::btree::{BehaviorStatus, BehaviorTreeNode, Sequence};
use hecs::World;
use std::collections::HashMap;

pub fn do_nothing() -> Box<dyn BehaviorTreeNode> {
    Box::new(DoNothing {})
}

pub fn find_food() -> Box<Sequence> {
    Box::new(Sequence::of(vec![
        FindNearestFood::new(),
        // move to food
        // pick up
        // consume
    ]))
}

struct DoNothing {}

impl BehaviorTreeNode for DoNothing {
    fn run(
        &mut self,
        _knowledge: &mut HashMap<String, String>,
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
        _knowledge: &mut HashMap<String, String>,
        _world: &mut World,
    ) -> BehaviorStatus {
        Success
    }
}
