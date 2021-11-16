use crate::no_such_entity::NoSuchEntity;
use crate::Entities;
use crate::Entity;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// A storage type based on a HashMap, intended for sparsely used components.
#[derive(Debug)]
pub struct MapStorage<T> {
    map: HashMap<u32, T>,
    entities: Arc<RwLock<Entities>>,
}

impl<T> MapStorage<T> {
    /// Create a new MapStorage<T>.
    pub fn new(entity_allocator: Arc<RwLock<Entities>>) -> Self {
        Self {
            map: HashMap::new(),
            entities: entity_allocator,
        }
    }

    /// Get a reference to the associated component for the given entity, if any.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            self.map.get(&entity.index)
        } else {
            None
        }
    }

    /// Get a mutable reference to the associated component for the given entity, if any.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            self.map.get_mut(&entity.index)
        } else {
            None
        }
    }

    /// Set the component for the given entity.
    /// Returns Err(NoSuchEnitty) if the given entity doesn't exist.
    /// Otherwise, returns the previous data stored in self for the given entity.
    pub fn set(&mut self, entity: Entity, data: T) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            Ok(self.map.insert(entity.index, data))
        } else {
            Err(NoSuchEntity {})
        }
    }

    /// Remove the component for the given entity.
    /// Returns the previous data associated with the given entity in self.
    /// Does not check if the entity exists; only use this if you know it exists, e.g.
    /// through invariants in your code or because you retrieved this in a loop iterating
    /// over all alive entities.
    pub fn remove_unchecked(&mut self, entity: Entity) -> Option<T> {
        self.map.remove(&entity.index)
    }

    /// Remove the component for the given entity.
    /// Returns the previous data associated with the given entity in self.
    pub fn remove(&mut self, entity: Entity) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            Ok(self.map.remove(&entity.index))
        } else {
            Err(NoSuchEntity)
        }
    }

    /// Remove the data stored in self for all entities.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct MapTestData(i32);

    #[test]
    fn map_get_not_set() {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity = Entity {
            index: 0,
            generation: 0,
        };
        let entry = map.get(entity);
        assert_eq!(entry, None);
    }

    #[test]
    fn map_get() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(entity, data)?;
        assert_eq!(old_data, None);
        let entry = map.get(entity);
        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn map_set_exists() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(entity, data)?;
        assert_eq!(old_data, None);
        let entity = Entity {
            index: 0,
            generation: 1,
        };
        let no_such_entity = map.set(entity, data);
        assert!(no_such_entity.is_err());
        Ok(())
    }

    #[test]
    fn remove_missing_is_ok() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let no_data = map.remove(entity)?;
        assert_eq!(no_data, None);
        Ok(())
    }

    #[test]
    fn can_insert_after_remove() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(entity, data)?;
        assert_eq!(old_data, None);
        let old_data = map.remove(entity)?;
        assert_eq!(old_data, Some(MapTestData(42)));

        let missing_entry = map.get(entity);
        assert_eq!(missing_entry, None);

        let entity = {
            let mut lock = entities.write().unwrap();
            lock.despawn(entity)?;
            lock.spawn()
        };

        let data = MapTestData(17);
        let old_data = map.set(entity, data)?;
        assert_eq!(old_data, None);
        let entry = map.get(entity);

        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn map_iter() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let entity_a = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(entity_a, MapTestData(17))?;
        assert_eq!(old_data, None);
        let entity_b = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(entity_b, MapTestData(42))?;
        assert_eq!(old_data, None);

        let entity_c = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(entity_c, MapTestData(123))?;
        assert_eq!(old_data, None);
        let old_data = map.remove(entity_c)?; //to get a None value which will be filtered out.
        assert_eq!(old_data, Some(MapTestData(123)));

        let mut v = entities
            .read()
            .unwrap()
            .iter()
            .map(|entity| (entity, map.get(entity)))
            .filter(|(_entity, data)| data.is_some())
            .collect::<Vec<_>>();
        v.sort_by(|(entity_a, _a), (entity_b, _b)| entity_a.index.cmp(&entity_b.index));
        assert_eq!(
            v,
            vec![
                (entity_a, Some(&MapTestData(17))),
                (entity_b, Some(&MapTestData(42)))
            ]
        );
        Ok(())
    }
}
