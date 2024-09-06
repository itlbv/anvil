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
    action_status: Option<BehaviorStatus>,
}

impl DoUntil {
    pub fn new(
        condition: Box<dyn BehaviorTreeNode>,
        action: Box<dyn BehaviorTreeNode>,
    ) -> Box<Self> {
        Box::new(Self {
            condition,
            action,
            action_status: None,
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
        match self.action_status.as_ref() {
            None => {}
            Some(status) => {
                match status {
                    Running => {
                        // actions are still running, let them continue and return Running
                        self.action_status =
                            Option::from(self.action.run(knowledge, entity_commands, registry));
                        return Running;
                    }
                    Success => {
                        println!("DoUntil action status is success")
                    }
                    _ => {}
                }
            }
        }

        // run condition check
        let condition_status = self.condition.run(knowledge, entity_commands, registry);
        match condition_status {
            Success => {
                // if condition success, return success
                println!("DoUntil condition success!");
                Success
            }
            Failure => {
                // if condition not success, run action, remember prev running status
                println!("DoUntil condition failure! Trying actions again");
                self.action_status =
                    Option::from(self.action.run(knowledge, entity_commands, registry));
                Running
            }
            Running => {
                // if condition not success, run action, remember prev running status
                println!("DoUntil running! Running actions");
                self.action_status =
                    Option::from(self.action.run(knowledge, entity_commands, registry));
                Running
            }
        }
    }
}

pub struct Sequence {
    name: String,
    children: Vec<Box<dyn BehaviorTreeNode>>,
    running_behavior_idx: i32,
}

impl Sequence {
    pub fn of(name: &str, children: Vec<Box<dyn BehaviorTreeNode>>) -> Box<Self> {
        Box::new(Self {
            name: String::from(name),
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
        println!("Running {} sequence", self.name);
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
        println!("{} sequence successful!", self.name);
        self.running_behavior_idx = 0; // reset idx to 0 to start anew
        Success
    }
}
