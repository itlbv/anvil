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

pub struct DoUntil {
    condition: Box<dyn BehaviorTreeNode>,
    action: Box<dyn BehaviorTreeNode>,
    status_previous_run: Option<BehaviorStatus>,
}

impl DoUntil {
    pub fn new(
        condition: Box<dyn BehaviorTreeNode>,
        action: Box<dyn BehaviorTreeNode>,
    ) -> Box<Self> {
        Box::new(Self {
            condition,
            action,
            status_previous_run: None,
        })
    }
}

impl BehaviorTreeNode for DoUntil {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        // if prev running status running, proceed to action
        match self.status_previous_run.as_ref() {
            None => {}
            Some(status) => {
                match status {
                    Running => {
                        // actions are still running, let them continue and return Running
                        self.action.run(knowledge, entity_commands, registry);
                        return Running;
                    }
                    _ => {}
                    _ => {}
                }
            }
        }

        // run condition check
        let status = self.condition.run(knowledge, entity_commands, registry);
        match status {
            Success => Success, // if condition success, return success
            Failure => {
                // if condition not success, run action, remember prev running status
                self.status_previous_run =
                    Option::from(self.action.run(knowledge, entity_commands, registry));
                Failure
            }
            Running => {
                // if condition not success, run action, remember prev running status
                self.status_previous_run =
                    Option::from(self.action.run(knowledge, entity_commands, registry));
                Running
            }
        }
    }
}

pub struct Sequence {
    children: Vec<Box<dyn BehaviorTreeNode>>,
    running_behavior_idx: i32,
}

impl Sequence {
    pub fn of(children: Vec<Box<dyn BehaviorTreeNode>>) -> Box<Self> {
        Box::new(Self {
            children,
            running_behavior_idx: -1,
        })
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
                    self.running_behavior_idx = i as i32;
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
