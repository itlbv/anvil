use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::btree::{BehaviorStatus, BehaviorTreeNode, Sequence};
use crate::components::StateType::{IDLE, MOVE};
use crate::components::{Food, Movement, Position, State};
use crate::entity_commands::EntityCommand;
use crate::entity_commands::EntityCommandType::RemoveFromMap;
use crate::{entity_commands, recipes, Knowledge, Recipe};
use hecs::{Component, World as ComponentRegistry};
use std::any::TypeId;
use std::collections::HashMap;

pub fn do_nothing() -> Box<dyn BehaviorTreeNode> {
    Box::new(DoNothing {})
}

pub fn build_house() -> Box<Sequence> {
    Sequence::of(vec![
        // choose recipe
        ChooseRecipe::new(),
        // DoUntil(HasAllForRecipe(), FindIngredientsForRecipeSequence())
        FindItem::new(),
        // find and reserve place
        // gather resources
        // move to position
        // build
    ])
}

struct ChooseRecipe {}

impl ChooseRecipe {
    fn new() -> Box<Self> {
        Box::new(ChooseRecipe {})
    }
}

impl BehaviorTreeNode for ChooseRecipe {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        knowledge.recipe = Option::from(recipes::house());
        Success
    }
}

struct FindItem {}

impl FindItem {
    fn new() -> Box<Self> {
        Box::new(FindItem {})
    }
}

impl BehaviorTreeNode for FindItem {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        match &knowledge.recipe {
            None => {
                println!("No recipe set!");
                Failure
            }
            Some(recipe) => {
                for ingredient in &recipe.ingredients {
                    let type_id = ingredient.0.to_owned();
                    if type_id == TypeId::of::<Food>() {
                        find_item::<Food>(registry);
                    }
                }
                Success
            }
        }
    }
}

fn find_item<T: Component>(registry: &mut ComponentRegistry) {
    for (item_entity, item) in registry.query_mut::<&T>() {
        println!("Found item");
    }
}

pub fn find_food() -> Box<Sequence> {
    Sequence::of(vec![
        FindNearestFood::new(),
        MoveToTarget::new(),
        PickUpTargetToInventory::new(),
        // consume
    ])
}

pub fn move_to_position() -> Box<dyn BehaviorTreeNode> {
    Box::new(MoveToPosition {})
}

pub fn move_to_target() -> Box<dyn BehaviorTreeNode> {
    Box::new(MoveToTarget {})
}

struct PickUpTargetToInventory {}

impl PickUpTargetToInventory {
    fn new() -> Box<Self> {
        Box::new(PickUpTargetToInventory {})
    }
}

impl BehaviorTreeNode for PickUpTargetToInventory {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        entity_commands: &mut Vec<EntityCommand>,
        _registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        // if no target is set, fail
        if knowledge.target.is_none() {
            println!("Target is not set, cannot execute PickUp!");
            return Failure;
        }

        // add target to inventory
        knowledge.inventory.push(knowledge.target.unwrap());

        // dispatch command to remove entity from map
        entity_commands::push_new_command(
            entity_commands,
            knowledge.target.unwrap(),
            RemoveFromMap,
        );

        Success
    }
}

struct DoNothing {}

impl BehaviorTreeNode for DoNothing {
    fn run(
        &mut self,
        _: &mut Knowledge,
        _: &mut Vec<EntityCommand>,
        _: &mut ComponentRegistry,
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
        _entity_commands: &mut Vec<EntityCommand>,
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
            None => {
                println!("Can't find food!");
                Failure
            }
            Some(target_entity) => {
                knowledge.target = Option::from(target_entity);
                println!("Finished finding food");
                Success
            }
        }
    }
}

struct MoveToPosition {}

impl MoveToPosition {
    fn new() -> Box<Self> {
        Box::new(MoveToPosition {})
    }
}

impl BehaviorTreeNode for MoveToPosition {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        _entity_commands: &mut Vec<EntityCommand>,
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
            println!("Finished moving to position");
            return Success;
        }

        // start movement
        state.state = MOVE;
        movement.destination_x = knowledge.destination_x;
        movement.destination_y = knowledge.destination_y;
        movement.distance = 0.1;

        Running
    }
}

struct MoveToTarget {}

impl MoveToTarget {
    fn new() -> Box<Self> {
        Box::new(MoveToTarget {})
    }
}

impl BehaviorTreeNode for MoveToTarget {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        _entity_commands: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        // check if target is set
        if knowledge.target.is_none() {
            println!("Target is not set, cannot execute MoveToEntity!");
            return Failure;
        }

        let own_pos = registry.get::<&Position>(knowledge.own_id).unwrap();
        let target_pos = registry
            .get::<&Position>(knowledge.target.unwrap())
            .unwrap();
        let mut movement = registry.get::<&mut Movement>(knowledge.own_id).unwrap();
        let mut state = registry.get::<&mut State>(knowledge.own_id).unwrap();

        // check if already arrived
        if (own_pos.x - target_pos.x).abs() < movement.distance
            && (own_pos.y - target_pos.y).abs() < movement.distance
        {
            state.state = IDLE;
            println!("Finished moving to target");
            return Success;
        }

        // start movement
        state.state = MOVE;
        movement.destination_x = target_pos.x;
        movement.destination_y = target_pos.y;
        movement.distance = 0.5;

        Running
    }
}
