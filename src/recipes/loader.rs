use super::types::{Recipe, RecipeIndex};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn load_dir(dir: &Path) -> Result<RecipeIndex> {
    let mut idx = RecipeIndex::default();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {:?}", dir))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            let text =
                fs::read_to_string(&path).with_context(|| format!("read_to_string {:?}", path))?;
            let list: Vec<Recipe> =
                ron::from_str(&text).with_context(|| format!("RON parse {:?}", path))?;
            for r in list {
                idx.insert(r);
            }
        }
    }
    Ok(idx)
}
