use blake3::Hasher;

/// Produce a deterministic 64-bit hash for the whole world.
/// IMPORTANT: iterate entities in a **canonical order** (e.g., by EntityId).
pub fn world_hash<F>(mut write_entity_bytes_in_order: F) -> u64
where
    F: FnMut(&mut Hasher),
{
    let mut h = Hasher::new();
    write_entity_bytes_in_order(&mut h);
    let bytes = h.finalize();
    // take first 8 bytes as u64
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes.as_bytes()[..8]);
    u64::from_le_bytes(arr)
}
