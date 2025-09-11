use crate::components::{Stone, Wood};
use std::any::TypeId;

pub struct Recipe {
    pub(crate) ingredients: Vec<(TypeId, usize)>,
}

pub fn house() -> Recipe {
    Recipe {
        ingredients: vec![(TypeId::of::<Wood>(), 1), (TypeId::of::<Stone>(), 1)],
    }
}
