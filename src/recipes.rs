use crate::components::{Stone, Wood};
use std::any::TypeId;
use std::collections::HashMap;

pub struct Recipe {
    pub(crate) ingredients: HashMap<TypeId, usize>,
}

pub fn house() -> Recipe {
    let mut ingredients = HashMap::new();
    ingredients.insert(TypeId::of::<Wood>(), 1);
    ingredients.insert(TypeId::of::<Stone>(), 1);
    Recipe { ingredients }
}
