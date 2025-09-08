use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub type RunSeed = u64;

/// Global, run-scoped seed. Keep it immutable for the whole run.
pub struct RngRun {
    pub seed: RunSeed,
}

impl RngRun {
    pub fn new(seed: RunSeed) -> Self {
        Self { seed }
    }
}

/// Deterministic RNG for a given (tick, stream).
/// `stream` lets you isolate domains (e.g., "ai", "spawns", "fx") to avoid cross-talk.
pub fn rng_for_tick(run: &RngRun, tick: u64, stream: u64) -> ChaCha8Rng {
    // Derive a 64-bit seed by hashing (run.seed, tick, stream) in a simple reversible way.
    // Avoid platform-dependent hashes here.
    let s =
        run.seed ^ tick.wrapping_mul(0x9E3779B97F4A7C15) ^ stream.wrapping_mul(0xD2B74407B1CE6E93);
    ChaCha8Rng::seed_from_u64(s)
}

/// Optional: per-entity RNG (order-independent) seeded by stable entity id
pub fn rng_for_entity(run: &RngRun, entity_id: u32, stream: u64) -> ChaCha8Rng {
    let s = run.seed
        ^ (entity_id as u64).wrapping_mul(0xA24BAED4963EE407)
        ^ stream.wrapping_mul(0x9E3779B185EBCA87);
    ChaCha8Rng::seed_from_u64(s)
}
