use blake3::Hasher;
use hecs::World as ComponentRegistry;

use crate::components::{Food, Hunger, Position, Shape, State, StateType, Stone, Wood};

#[inline]
fn f32_bits_canonical(x: f32) -> u32 {
    // IEEE754 to bits, canonicalize -0.0 and all NaNs to a single quiet NaN
    let mut b = x.to_bits();
    if b == 0x8000_0000 {
        b = 0;
    } // -0.0 -> +0.0
    let is_nan = (b & 0x7f80_0000) == 0x7f80_0000 && (b & 0x007f_ffff) != 0;
    if is_nan {
        b = 0x7fc0_0000;
    } // canonical quiet NaN
    b
}

#[inline]
fn feed_tag(hasher: &mut Hasher, tag: &[u8; 4]) {
    hasher.update(tag);
}
#[inline]
fn u64_le(x: u64) -> [u8; 8] {
    x.to_le_bytes()
}
#[inline]
fn u32_le(x: u32) -> [u8; 4] {
    x.to_le_bytes()
}

fn state_to_u8(s: &StateType) -> u8 {
    match s {
        StateType::Idle => 0,
        StateType::Move => 1,
        // keep numbering stable if you extend the enum
    }
}

#[inline]
fn finish64(mut hasher: Hasher) -> u64 {
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(out)
}

#[cfg(feature = "hash_debug")]
#[derive(Debug, Clone, Copy)]
pub struct WorldHashBreakdown {
    pub total: u64,
    pub pos: u64,
    pub hun: u64,
    pub sta: u64,
    pub food: u64,
    pub wood: u64,
    pub stone: u64,
    pub shape: u64,
}

#[cfg(feature = "hash_debug")]
pub fn world_hash_breakdown(registry: &ComponentRegistry) -> WorldHashBreakdown {
    let mut total_hasher = Hasher::new();

    // POS
    let pos = {
        let mut rows: Vec<(u64, u32, u32)> = Vec::new();
        for (e, pos) in registry.query::<&Position>().iter() {
            rows.push((
                e.to_bits().get(),
                f32_bits_canonical(pos.x),
                f32_bits_canonical(pos.y),
            ));
        }
        rows.sort_unstable_by_key(|r| r.0);

        let mut h = Hasher::new();
        feed_tag(&mut h, b"POS\0");
        feed_tag(&mut total_hasher, b"POS\0");
        for (id, x, y) in rows {
            let idb = u64_le(id);
            let xb = u32_le(x);
            let yb = u32_le(y);
            h.update(&idb);
            h.update(&xb);
            h.update(&yb);
            total_hasher.update(&idb);
            total_hasher.update(&xb);
            total_hasher.update(&yb);
        }
        finish64(h)
    };

    // HUN (only value; no Instants)
    let hun = {
        let mut rows: Vec<(u64, u32)> = Vec::new();
        for (e, h) in registry.query::<&Hunger>().iter() {
            rows.push((e.to_bits().get(), f32_bits_canonical(h.value as f32)));
        }
        rows.sort_unstable_by_key(|r| r.0);

        let mut h = Hasher::new();
        feed_tag(&mut h, b"HUN\0");
        feed_tag(&mut total_hasher, b"HUN\0");
        for (id, vb) in rows {
            let idb = u64_le(id);
            let vbb = u32_le(vb);
            h.update(&idb);
            h.update(&vbb);
            total_hasher.update(&idb);
            total_hasher.update(&vbb);
        }
        finish64(h)
    };

    // STA
    let sta = {
        let mut rows: Vec<(u64, u8)> = Vec::new();
        for (e, s) in registry.query::<&State>().iter() {
            rows.push((e.to_bits().get(), state_to_u8(&s.state)));
        }
        rows.sort_unstable_by_key(|r| r.0);

        let mut h = Hasher::new();
        feed_tag(&mut h, b"STA\0");
        feed_tag(&mut total_hasher, b"STA\0");
        for (id, st) in rows {
            let idb = u64_le(id);
            h.update(&idb);
            h.update(&[st]);
            total_hasher.update(&idb);
            total_hasher.update(&[st]);
        }
        finish64(h)
    };

    // FOOD
    let food = {
        let mut ids: Vec<u64> = registry
            .query::<&Food>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();

        let mut h = Hasher::new();
        feed_tag(&mut h, b"FOOD");
        feed_tag(&mut total_hasher, b"FOOD");
        for id in ids {
            let idb = u64_le(id);
            h.update(&idb);
            total_hasher.update(&idb);
        }
        finish64(h)
    };

    // WOOD
    let wood = {
        let mut ids: Vec<u64> = registry
            .query::<&Wood>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();

        let mut h = Hasher::new();
        feed_tag(&mut h, b"WOOD");
        feed_tag(&mut total_hasher, b"WOOD");
        for id in ids {
            let idb = u64_le(id);
            h.update(&idb);
            total_hasher.update(&idb);
        }
        finish64(h)
    };

    // STONE
    let stone = {
        let mut ids: Vec<u64> = registry
            .query::<&Stone>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();

        let mut h = Hasher::new();
        feed_tag(&mut h, b"STON");
        feed_tag(&mut total_hasher, b"STON");
        for id in ids {
            let idb = u64_le(id);
            h.update(&idb);
            total_hasher.update(&idb);
        }
        finish64(h)
    };

    // SHAPE (visuals)
    let shape = {
        let mut rows: Vec<(u64, u32, u32, u8, u8, u8, u8)> = Vec::new();
        for (e, sh) in registry.query::<&Shape>().iter() {
            let (r, g, b, a) = sh.color;
            rows.push((
                e.to_bits().get(),
                f32_bits_canonical(sh.width),
                f32_bits_canonical(sh.height),
                r,
                g,
                b,
                a,
            ));
        }
        rows.sort_unstable_by_key(|r| r.0);

        let mut h = Hasher::new();
        feed_tag(&mut h, b"SHP\0");
        feed_tag(&mut total_hasher, b"SHP\0");
        for (id, w, hgt, r, g, b, a) in rows {
            let idb = u64_le(id);
            let wb = u32_le(w);
            let hb = u32_le(hgt);
            h.update(&idb);
            h.update(&wb);
            h.update(&hb);
            h.update(&[r, g, b, a]);
            total_hasher.update(&idb);
            total_hasher.update(&wb);
            total_hasher.update(&hb);
            total_hasher.update(&[r, g, b, a]);
        }
        finish64(h)
    };

    WorldHashBreakdown {
        total: finish64(total_hasher),
        pos,
        hun,
        sta,
        food,
        wood,
        stone,
        shape,
    }
}

/// Returns the same total world hash you’ve been using (no breakdown needed).
pub fn world_hash(registry: &ComponentRegistry) -> u64 {
    let mut hasher = Hasher::new();

    // POS
    {
        let mut rows: Vec<(u64, u32, u32)> = Vec::new();
        for (e, pos) in registry.query::<&Position>().iter() {
            rows.push((
                e.to_bits().get(),
                f32_bits_canonical(pos.x),
                f32_bits_canonical(pos.y),
            ));
        }
        rows.sort_unstable_by_key(|r| r.0);
        feed_tag(&mut hasher, b"POS\0");
        for (id, x, y) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(x));
            hasher.update(&u32_le(y));
        }
    }

    // HUN
    {
        let mut rows: Vec<(u64, u32)> = Vec::new();
        for (e, h) in registry.query::<&Hunger>().iter() {
            rows.push((e.to_bits().get(), f32_bits_canonical(h.value as f32)));
        }
        rows.sort_unstable_by_key(|r| r.0);
        feed_tag(&mut hasher, b"HUN\0");
        for (id, vb) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(vb));
        }
    }

    // STA
    {
        let mut rows: Vec<(u64, u8)> = Vec::new();
        for (e, s) in registry.query::<&State>().iter() {
            rows.push((e.to_bits().get(), state_to_u8(&s.state)));
        }
        rows.sort_unstable_by_key(|r| r.0);
        feed_tag(&mut hasher, b"STA\0");
        for (id, st) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&[st]);
        }
    }

    // FOOD / WOOD / STONE
    {
        let mut ids: Vec<u64> = registry
            .query::<&Food>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();
        feed_tag(&mut hasher, b"FOOD");
        for id in ids {
            hasher.update(&u64_le(id));
        }
    }
    {
        let mut ids: Vec<u64> = registry
            .query::<&Wood>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();
        feed_tag(&mut hasher, b"WOOD");
        for id in ids {
            hasher.update(&u64_le(id));
        }
    }
    {
        let mut ids: Vec<u64> = registry
            .query::<&Stone>()
            .iter()
            .map(|(e, _)| e.to_bits().get())
            .collect();
        ids.sort_unstable();
        feed_tag(&mut hasher, b"STON");
        for id in ids {
            hasher.update(&u64_le(id));
        }
    }

    // SHAPE
    {
        let mut rows: Vec<(u64, u32, u32, u8, u8, u8, u8)> = Vec::new();
        for (e, sh) in registry.query::<&Shape>().iter() {
            let (r, g, b, a) = sh.color;
            rows.push((
                e.to_bits().get(),
                f32_bits_canonical(sh.width),
                f32_bits_canonical(sh.height),
                r,
                g,
                b,
                a,
            ));
        }
        rows.sort_unstable_by_key(|r| r.0);
        feed_tag(&mut hasher, b"SHP\0");
        for (id, w, hgt, r, g, b, a) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(w));
            hasher.update(&u32_le(hgt));
            hasher.update(&[r, g, b, a]);
        }
    }

    finish64(hasher)
}
