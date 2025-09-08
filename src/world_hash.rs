use blake3::Hasher;
use hecs::World as ComponentRegistry;

pub fn world_hash(world: &ComponentRegistry) -> u64 {
    let mut h = Hasher::new();

    // --- Hunger ---
    {
        let mut rows: Vec<(u64, u8, f32)> = Vec::new();
        for (e, hunger) in world.query::<&crate::components::Hunger>().iter() {
            rows.push((e.to_bits().get(), hunger.value, hunger.acc_seconds));
        }
        rows.sort_unstable_by_key(|(id, _, _)| *id);
        for (id, v, acc) in rows {
            h.update(&id.to_le_bytes());
            h.update(&[v]);
            h.update(&acc.to_le_bytes());
        }
    }

    // --- Position (example) ---
    {
        let mut rows: Vec<(u64, f32, f32)> = Vec::new();
        for (e, pos) in world.query::<&crate::components::Position>().iter() {
            rows.push((e.to_bits().get(), pos.x, pos.y));
        }
        rows.sort_unstable_by_key(|(id, _, _)| *id);
        for (id, x, y) in rows {
            h.update(&id.to_le_bytes());
            h.update(&x.to_le_bytes());
            h.update(&y.to_le_bytes());
        }
    }

    let bytes = h.finalize();
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes.as_bytes()[..8]);
    u64::from_le_bytes(arr)
}
