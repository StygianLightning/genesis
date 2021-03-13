use crate::no_such_entity::NoSuchEntity;
use crate::Entities;
use crate::Entity;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct MapStorage<T> {
    map: HashMap<u32, Option<T>>,
    entities: Arc<RwLock<Entities>>,
}

impl<T> MapStorage<T> {
    pub fn new(id_allocator: Arc<RwLock<Entities>>) -> Self {
        Self {
            map: HashMap::new(),
            entities: id_allocator,
        }
    }

    pub fn get(&self, id: Entity) -> Option<&T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            self.map.get(&id.index).unwrap_or(&None).as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Entity) -> Option<&mut T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            if let Some(entry) = self.map.get_mut(&id.index) {
                entry.as_mut()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set(&mut self, id: Entity, data: T) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            let entry = self.map.entry(id.index);
            Ok(entry.or_default().replace(data))
        } else {
            Err(NoSuchEntity {})
        }
    }

    pub fn remove(&mut self, id: Entity) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            if let Some(entry) = self.map.get_mut(&id.index) {
                Ok(entry.take())
            } else {
                Ok(None)
            }
        } else {
            Err(NoSuchEntity)
        }
    }

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
        let id = Entity {
            index: 0,
            generation: 0,
        };
        let entry = map.get(id);
        assert_eq!(entry, None);
    }

    #[test]
    fn map_get() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(id, data)?;
        assert_eq!(old_data, None);
        let entry = map.get(id);
        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn map_set_exists() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(id, data)?;
        assert_eq!(old_data, None);
        let id = Entity {
            index: 0,
            generation: 1,
        };
        let no_such_entity = map.set(id, data);
        assert!(no_such_entity.is_err());
        Ok(())
    }

    #[test]
    fn remove_missing_is_ok() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let no_data = map.remove(id)?;
        assert_eq!(no_data, None);
        Ok(())
    }

    #[test]
    fn can_insert_after_remove() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = MapTestData(42);
        let old_data = map.set(id, data)?;
        assert_eq!(old_data, None);
        let old_data = map.remove(id)?;
        assert_eq!(old_data, Some(MapTestData(42)));

        let missing_entry = map.get(id);
        assert_eq!(missing_entry, None);

        let id = {
            let mut lock = entities.write().unwrap();
            lock.despawn(id)?;
            lock.spawn()
        };

        let data = MapTestData(17);
        let old_data = map.set(id, data)?;
        assert_eq!(old_data, None);
        let entry = map.get(id);

        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn map_iter() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut map = MapStorage::<MapTestData>::new(Arc::clone(&entities));
        let id_a = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(id_a, MapTestData(17))?;
        assert_eq!(old_data, None);
        let id_b = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(id_b, MapTestData(42))?;
        assert_eq!(old_data, None);

        let id_c = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let old_data = map.set(id_c, MapTestData(123))?;
        assert_eq!(old_data, None);
        let old_data = map.remove(id_c)?; //to get a None value which will be filtered out.
        assert_eq!(old_data, Some(MapTestData(123)));

        let mut v = entities
            .read()
            .unwrap()
            .iter()
            .map(|id| (id, map.get(id)))
            .filter(|(_id, data)| data.is_some())
            .collect::<Vec<_>>();
        v.sort_by(|(idx_a, _a), (idx_b, _b)| idx_a.index.cmp(&idx_b.index));
        assert_eq!(
            v,
            vec![
                (id_a, Some(&MapTestData(17))),
                (id_b, Some(&MapTestData(42)))
            ]
        );
        Ok(())
    }
}
