use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProductKind {
    Item,
    Building,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub kind: ProductKind,
    pub id: String,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub kind: ProductKind,
    pub id: String,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub product: Product,
    pub ingredients: Vec<Ingredient>,
    pub tools: Vec<String>,
    pub time_ms: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub flags: u32,
}

pub type RecipeId = u32;

#[derive(Default)]
pub struct RecipeIndex {
    by_product: std::collections::HashMap<String, Vec<RecipeId>>,
    recipes: Vec<Recipe>,
}

impl RecipeIndex {
    pub fn recipes_for(&self, product_id: &str) -> &[RecipeId] {
        static EMPTY: Vec<RecipeId> = Vec::new();
        self.by_product
            .get(product_id)
            .map(|v| v.as_slice())
            .unwrap_or(&EMPTY)
    }
    pub fn get(&self, id: RecipeId) -> &Recipe {
        &self.recipes[id as usize]
    }
    pub fn iter(&self) -> impl Iterator<Item = (RecipeId, &Recipe)> {
        self.recipes
            .iter()
            .enumerate()
            .map(|(i, r)| (i as RecipeId, r))
    }
    pub fn insert(&mut self, r: Recipe) {
        let id = self.recipes.len() as RecipeId;
        self.by_product
            .entry(r.product.id.clone())
            .or_default()
            .push(id);
        self.recipes.push(r);
    }
}
