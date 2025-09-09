use hecs::Entity;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ---- T = Entity -------------------------------------------------------------
pub fn serialize<S>(e: &Entity, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // to_bits() -> NonZero<u64> in hecs 0.10.5
    s.serialize_u64(e.to_bits().get())
}

pub fn deserialize<'de, D>(d: D) -> Result<Entity, D::Error>
where
    D: Deserializer<'de>,
{
    let bits: u64 = u64::deserialize(d)?;
    Entity::from_bits(bits)
        .ok_or_else(|| de::Error::custom("invalid entity bits (zero or out of range)".to_string()))
}

// ---- T = Option<Entity> -----------------------------------------------------
pub mod opt {
    use super::*;
    pub fn serialize<S>(e: &Option<Entity>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match e {
            Some(ent) => s.serialize_some(&ent.to_bits().get()),
            None => s.serialize_none(),
        }
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Option<Entity>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_bits: Option<u64> = Option::<u64>::deserialize(d)?;
        opt_bits
            .map(|bits| {
                Entity::from_bits(bits).ok_or_else(|| {
                    de::Error::custom("invalid entity bits (zero or out of range)".to_string())
                })
            })
            .transpose()
    }
}

// ---- T = Vec<Entity> --------------------------------------------------------
pub mod vec {
    use super::*;
    pub fn serialize<S>(v: &Vec<Entity>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let tmp: Vec<u64> = v.iter().map(|e| e.to_bits().get()).collect();
        tmp.serialize(s)
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Vec<Entity>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits_vec: Vec<u64> = Vec::<u64>::deserialize(d)?;
        bits_vec
            .into_iter()
            .map(|bits| {
                Entity::from_bits(bits).ok_or_else(|| {
                    de::Error::custom("invalid entity bits (zero or out of range)".to_string())
                })
            })
            .collect()
    }
}
