use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::hash::Hash;

pub fn serialize<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize + Clone,
    V: Serialize + Clone,
    S: Serializer,
{
    let vec: Vec<(K, V)> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    vec.serialize(serializer)
}

pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let vec: Vec<(K, V)> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}

use crate::core::piece::PieceKind;
use crate::core::PlayerId;

pub fn serialize_hand<S>(
    hand: &HashMap<PlayerId, HashMap<PieceKind, usize>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec: Vec<(PlayerId, Vec<(PieceKind, usize)>)> = hand
        .iter()
        .map(|(p, h)| {
            let h_vec: Vec<(PieceKind, usize)> = h.iter().map(|(k, v)| (*k, *v)).collect();
            (*p, h_vec)
        })
        .collect();
    vec.serialize(serializer)
}

pub fn deserialize_hand<'de, D>(
    deserializer: D,
) -> Result<HashMap<PlayerId, HashMap<PieceKind, usize>>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<(PlayerId, Vec<(PieceKind, usize)>)> = Vec::deserialize(deserializer)?;
    let mut hand = HashMap::new();
    for (p, h_vec) in vec {
        hand.insert(p, h_vec.into_iter().collect());
    }
    Ok(hand)
}
