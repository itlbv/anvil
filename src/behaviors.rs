use crate::btree::BehaviorStatus::{Failure, Running, Success};
use crate::btree::{BehaviorStatus, BehaviorTreeNode, DoUntil, Sequence};
use crate::components::StateType::{Idle, Move};
use crate::components::{Food, Movement, Position, State, Stone, Wood};
use crate::entity_commands::EntityCommand;
use crate::entity_commands::EntityCommandType::RemoveFromMap;
use crate::{entity_commands, recipes, EntityWithType, Knowledge};
use hecs::{Component, Entity, World as ComponentRegistry};
use std::any::TypeId;
use std::collections::HashMap;

pub fn do_nothing() -> Box<dyn BehaviorTreeNode> {
    Box::new(DoNothing {})
}

pub fn build_house() -> Box<Sequence> {
    Sequence::of(
        "build_house",
        vec![
            ChooseRecipe::new(),
            // find and reserve place
            collect_items_from_recipe(),
            // move to position
            // build
        ],
    )
}

struct HasAllInRecipe {}

impl HasAllInRecipe {
    fn new() -> Box<Self> {
        Box::new(HasAllInRecipe {})
    }
}

impl BehaviorTreeNode for HasAllInRecipe {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        _entity_commands: &mut Vec<EntityCommand>,
        _registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        println!("HasAllInRecipe check!");
        match &knowledge.recipe {
            None => {
                println!("No recipe set! HasAllInRecipe failed");
                Failure
            }
            Some(recipe) => {
                for (item_type_id, count) in &recipe.ingredients {
                    if knowledge.inventory.get(item_type_id).is_none()
                        || knowledge.inventory.get(item_type_id).unwrap().len() < *count
                    {
                        println!("Some items from recipe not collected");
                        return Failure;
                    }
                }
                println!("Everything from recipe is collected");
                Success // everything is collected
            }
        }
    }
}

pub fn collect_items_from_recipe() -> Box<dyn BehaviorTreeNode> {
    DoUntil::new(
        HasAllInRecipe::new(),
        Sequence::of(
            "collect_items_for_recipe",
            vec![
                FindItemFromRecipe::new(),
                MoveToTarget::new(),
                PickUpTargetToInventory::new(),
            ],
        ),
    )
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
        _: &mut Vec<EntityCommand>,
        _: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        knowledge.recipe = Option::from(recipes::house());
        Success
    }
}

struct FindItemFromRecipe {}

impl FindItemFromRecipe {
    fn new() -> Box<Self> {
        Box::new(FindItemFromRecipe {})
    }
}

impl BehaviorTreeNode for FindItemFromRecipe {
    fn run(
        &mut self,
        knowledge: &mut Knowledge,
        _: &mut Vec<EntityCommand>,
        registry: &mut ComponentRegistry,
    ) -> BehaviorStatus {
        println!("FindItemFromRecipe");
        match &knowledge.recipe {
            None => {
                println!("No recipe set! FindItemFromRecipe failed");
                Failure
            }
            Some(recipe) => {
                for (item_type_id, count) in &recipe.ingredients {
                    let item_type_name = get_type_name(*item_type_id);
                    if knowledge.inventory.get(item_type_id).is_none()
                        || knowledge.inventory.get(item_type_id).unwrap().len() < *count
                    {
                        return match find_item_by_type_id(*item_type_id, registry) {
                            None => {
                                println!("Can't find item from recipe!");
                                Failure
                            }
                            Some(item) => {
                                println!("Found item, set target");
                                knowledge.target =
                                    Option::from(EntityWithType::new(*item_type_id, item));
                                Success
                            }
                        };
                    }
                }
                println!("Probably all items are collected in FindItemFromRecipe");
                Failure // no new items found so it should fail? if it is success, next moveTo behavior fails because target is not updated but target entity was despawned withouth position
            }
        }
    }
}

fn get_type_name(type_id: TypeId) -> String {
    if type_id == TypeId::of::<Food>() {
        return String::from("Food");
    } else if type_id == TypeId::of::<Wood>() {
        return String::from("Wood");
    } else if type_id == TypeId::of::<Stone>() {
        return String::from("Stone");
    }
    String::from("Unknown type")
}

fn find_item_by_type_id(type_id: TypeId, registry: &mut ComponentRegistry) -> Option<Entity> {
    if type_id == TypeId::of::<Food>() {
        return find_item::<Food>(registry);
    } else if type_id == TypeId::of::<Wood>() {
        return find_item::<Wood>(registry);
    } else if type_id == TypeId::of::<Stone>() {
        return find_item::<Stone>(registry);
    }
    None
}

fn find_item<T: Component>(registry: &mut ComponentRegistry) -> Option<Entity> {
    // find item with position
    for (entity, (_, _)) in registry.query_mut::<(&T, &Position)>() {
        return Some(entity);
    }
    None
}

pub fn find_food() -> Box<Sequence> {
    Sequence::of(
        "find_food",
        vec![
            FindNearestFood::new(),
            MoveToTarget::new(),
            PickUpTargetToInventory::new(),
            // consume
        ],
    )
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
        println!("PickUpTargetToInventory");
        // if no target is set, fail
        if knowledge.target.is_none() {
            println!("Target is not set, cannot PickUpTargetToInventory!");
            return Failure;
        }

        let target_with_type = knowledge.target.as_ref().unwrap();

        // add target to inventory
        add_item_to_inventory(
            &mut knowledge.inventory,
            target_with_type.type_id,
            target_with_type.entity,
        );

        // dispatch command to remove entity from map
        entity_commands::push_new_command(
            entity_commands,
            knowledge.target.as_ref().unwrap().entity,
            RemoveFromMap,
        );

        Success
    }
}

fn add_item_to_inventory(
    inventory: &mut HashMap<TypeId, Vec<Entity>>,
    type_id: TypeId,
    item: Entity,
) {
    match inventory.get_mut(&type_id) {
        None => {
            inventory.insert(type_id, vec![item]);
        }
        Some(entities) => {
            entities.push(item);
        }
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
        for (food_entity, (food, pos)) in registry.query_mut::<(&Food, &Position)>() {
            let dist_x = (pos.x - own_pos_x).abs();
            let dist_y = (pos.y - own_pos_y).abs();
            let dist = dist_x.hypot(dist_y);
            if dist < smallest_distance {
                smallest_distance = dist;
                nearest_food = Option::from(EntityWithType {
                    type_id: food.type_id,
                    entity: food_entity,
                });
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
            state.state = Idle;
            println!("Finished moving to position");
            return Success;
        }

        // start movement
        state.state = Move;
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
            println!("Target is not set, cannot execute MoveToTarget!");
            return Failure;
        }

        let target_entity = knowledge.target.as_ref().unwrap().entity;

        let own_pos = registry.get::<&Position>(knowledge.own_id).unwrap();
        let target_pos = registry.get::<&Position>(target_entity).unwrap();
        let mut movement = registry.get::<&mut Movement>(knowledge.own_id).unwrap();
        let mut state = registry.get::<&mut State>(knowledge.own_id).unwrap();

        // check if already arrived
        if (own_pos.x - target_pos.x).abs() < movement.distance
            && (own_pos.y - target_pos.y).abs() < movement.distance
        {
            state.state = Idle;
            println!("Finished moving to target");
            return Success;
        }

        // start movement
        state.state = Move;
        movement.destination_x = target_pos.x;
        movement.destination_y = target_pos.y;
        movement.distance = 0.5;

        Running
    }
}
