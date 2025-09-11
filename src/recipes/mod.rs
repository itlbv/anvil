pub mod loader;
pub mod types;

use std::path::Path;
use types::RecipeIndex;

pub struct RecipeDb(pub RecipeIndex);

impl RecipeDb {
    pub fn load_from_assets<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let idx = loader::load_dir(dir.as_ref())?;
        Ok(Self(idx))
    }
    pub fn recipes_for(&self, product_id: &str) -> &[types::RecipeId] {
        self.0.recipes_for(product_id)
    }
    pub fn get(&self, id: types::RecipeId) -> &types::Recipe {
        self.0.get(id)
    }
}
