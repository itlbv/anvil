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
        // add more variants here keeping numbers stable if you extend the enum
    }
}

pub fn world_hash(registry: &ComponentRegistry) -> u64 {
    let mut hasher = Hasher::new();

    // -----------------------
    // Positions (tag "POS\0")
    // -----------------------
    {
        let mut rows: Vec<(u64, u32, u32)> = Vec::new();
        for (e, pos) in registry.query::<&Position>().iter() {
            let id = e.to_bits().get(); // NonZero<u64> -> u64
            rows.push((id, f32_bits_canonical(pos.x), f32_bits_canonical(pos.y)));
        }
        rows.sort_unstable_by_key(|r| r.0);

        feed_tag(&mut hasher, b"POS\0");
        for (id, x, y) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(x));
            hasher.update(&u32_le(y));
        }
    }

    // -----------------------
    // Hunger (tag "HUN\0")
    // Hash ONLY the value (f32). Do NOT hash last_updated/Instant.
    // -----------------------
    {
        let mut rows: Vec<(u64, u32)> = Vec::new();
        for (e, h) in registry.query::<&Hunger>().iter() {
            let id = e.to_bits().get();
            // If Hunger.value is f32 now (per your refactor), hash its canonical bits.
            // If it’s still an integer in your code, cast to f32 or encode as u32/u8 as needed.
            #[allow(clippy::useless_conversion)]
            let v_bits = f32_bits_canonical(h.value as f32);
            rows.push((id, v_bits));
        }
        rows.sort_unstable_by_key(|r| r.0);

        feed_tag(&mut hasher, b"HUN\0");
        for (id, v_bits) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(v_bits));
        }
    }

    // -----------------------
    // State (tag "STA\0")
    // -----------------------
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

    // -----------------------
    // Static resources (presence) — Food/Wood/Stone (tags "FOOD", "WOOD", "STON")
    // -----------------------
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

    // -----------------------
    // Optional: Shape (tag "SHP\0") — only if you want visual diffs to affect hash
    // -----------------------
    #[allow(unused)]
    {
        let mut rows: Vec<(u64, u32, u32, u8, u8, u8, u8)> = Vec::new();
        for (e, sh) in registry.query::<&Shape>().iter() {
            // Assuming Shape { width: f32, height: f32, color: (u8,u8,u8,u8) }
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
        for (id, w, h, r, g, b, a) in rows {
            hasher.update(&u64_le(id));
            hasher.update(&u32_le(w));
            hasher.update(&u32_le(h));
            hasher.update(&[r, g, b, a]);
        }
    }

    // Finalize to u64 (little-endian of first 8 bytes)
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(out)
}
