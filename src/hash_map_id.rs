use std::collections::HashMap;

use serde::{Deserialize, Serialize};


pub type Id = u64;

#[derive(Serialize, Deserialize)]
pub struct HashMapId<V> {
    pub id: Id,
    pub hash_map: HashMap<Id, V>,
}

impl<V> HashMapId<V> {
    pub fn new() -> Self {
        HashMapId {
            id: 1,
            hash_map: HashMap::new(),
        }
    }

    pub fn reserve_id(&mut self) -> Id {
        let id = self.id;
        self.id += 1;
        id
    }

    pub fn insert(&mut self, value: V) -> Id {
        let id = self.id;
        self.id += 1;
        self.hash_map.insert(id, value);
        id
    }

    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, Id, V> {
        self.hash_map.values_mut()
    }
}