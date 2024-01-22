use crate::behavior::BehaviorStatus::{Failure, Running, Success};
use crate::components::Hunger;
use hecs::{Entity, World};

enum BehaviorStatus {
    Success,
    Failure,
    Running,
}

trait BehaviorTreeNode {
    fn run(&mut self) -> BehaviorStatus;
}

struct Sequence {
    children: Vec<Box<dyn BehaviorTreeNode>>,
    running_behavior_idx: i32,
}

impl Sequence {
    fn of(children: Vec<Box<dyn BehaviorTreeNode>>) -> Self {
        Self {
            children,
            running_behavior_idx: -1,
        }
    }
}

impl BehaviorTreeNode for Sequence {
    fn run(&mut self) -> BehaviorStatus {
        let mut i = 0;
        while i < self.children.len() {
            if self.running_behavior_idx >= 0 {
                i = self.running_behavior_idx as usize;
            }
            let status = self.children[i].run();
            match status {
                Failure => return Failure,
                Success => {
                    i += 1;
                }
                Running => {
                    self.running_behavior_idx = i as i32;
                    return Running;
                }
            };
        }
        Success
    }
}

pub struct Behavior {
    pub behavior_tree: Box<dyn BehaviorTreeNode>,
}

impl Behavior {
    pub fn new() -> Self {
        Self {
            behavior_tree: do_nothing(),
        }
    }

    pub fn run(&mut self, entity: Entity, world: &mut World) {
        let hunger = world.get::<&Hunger>(entity).unwrap();
        if hunger.value > 3 {
            self.behavior_tree = find_food();
        }
        self.behavior_tree.run();
    }
}

fn find_food() -> Box<Sequence> {
    Box::new(Sequence::of(vec![FindNearestFood::new()]))
}

struct FindNearestFood {}

impl FindNearestFood {
    fn new() -> Box<Self> {
        Box::new(FindNearestFood {})
    }
}

impl BehaviorTreeNode for FindNearestFood {
    fn run(&mut self) -> BehaviorStatus {
        Success
    }
}

fn do_nothing() -> Box<DoNothing> {
    Box::new(DoNothing {})
}

struct DoNothing {}

impl BehaviorTreeNode for DoNothing {
    fn run(&mut self) -> BehaviorStatus {
        Success
    }
}
