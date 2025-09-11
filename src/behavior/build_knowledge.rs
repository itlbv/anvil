use crate::recipes::{types::Recipe, RecipeDb};

/// Return all recipe definitions that can produce `product_id`.
/// Keeps planners ignorant of storage/index details.
pub fn query_recipes_for_build<'a>(
    product_id: &str,
    recipes: &'a RecipeDb,
) -> impl Iterator<Item = &'a Recipe> {
    recipes
        .recipes_for(product_id)
        .iter()
        .map(|rid| recipes.get(*rid))
}

/// Convenience: pick the first available recipe (simple default policy).
pub fn first_recipe_for<'a>(product_id: &str, recipes: &'a RecipeDb) -> Option<&'a Recipe> {
    query_recipes_for_build(product_id, recipes).next()
}
