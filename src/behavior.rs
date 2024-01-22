use crate::behavior::BehaviorStatus::{Failure, Running, Success};
use hecs::Entity;

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
        while i < self.children.len() - 1 {
            if self.running_behavior_idx >= 0 {
                i = self.running_behavior_idx as usize;
            }
            let status = self.children[i].run();
            return match status {
                Success => Success,
                Failure => Failure,
                Running => {
                    self.running_behavior_idx = i as i32;
                    Running
                }
            };
        }
        println!(
            "Returning Success in behavior outside matching logic. This should never be reachable"
        );
        Success
    }
}

pub struct Behavior {
    pub behavior_tree: Box<dyn BehaviorTreeNode>,
}

impl Behavior {
    pub fn new(&self, behavior: Box<dyn BehaviorTreeNode>) -> Self {
        Self {
            behavior_tree: behavior,
        }
    }

    pub fn run(&mut self, entity: &Entity) {
        self.behavior_tree.run();
    }
}
