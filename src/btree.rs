use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::entity_commands::EntityCommand;
use crate::Knowledge;

use hecs::World as ComponentRegistry;

pub enum BehaviorStatus {
    Success,
    Failure,
    Running,
}

pub trait BehaviorTreeNode {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus;
}

pub struct Sequence {
    children: Vec<Box<dyn BehaviorTreeNode>>,
    running_behavior_idx: i32,
}

impl Sequence {
    pub fn of(children: Vec<Box<dyn BehaviorTreeNode>>) -> Self {
        Self {
            children,
            running_behavior_idx: -1,
        }
    }
}

impl BehaviorTreeNode for Sequence {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        let mut i = 0;
        while i < self.children.len() {
            if self.running_behavior_idx >= 0 {
                i = self.running_behavior_idx as usize;
            }
            let status = self.children[i].run(knowledge, entity_commands, registry);
            match status {
                Failure => return Failure,
                Success => {
                    i += 1;
                    self.running_behavior_idx = i as i32
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
