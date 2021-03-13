use super::entity::Entity;
use crate::no_such_entity::NoSuchEntity;
use crate::Entities;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// A storage type that stores components in a contiguous Vec<T>.
#[derive(Debug)]
pub struct VecStorage<T> {
    vec: Vec<Option<T>>,
    entities: Arc<RwLock<Entities>>,
}

impl<T> VecStorage<T> {
    /// Create a new VecStorage<T> with the specified initial capacity.
    pub fn new(entities: Arc<RwLock<Entities>>, capacity: u32) -> Self {
        let mut vec = vec![];
        vec.resize_with(capacity as usize, Default::default);
        Self { vec, entities }
    }

    /// Get a reference to the component associated with the given entity in self, if any.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            self.vec
                .get(entity.index as usize)
                .unwrap_or(&None)
                .as_ref()
        } else {
            None
        }
    }

    /// Get a mutable reference to the component associated with the given entity in self, if any.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            if let Some(entry) = self.vec.get_mut(entity.index as usize) {
                entry.as_mut()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Set the component for the given entity.
    /// Returns Err(NoSuchEntity) if the given entity doesn't exist.
    /// Otherwise, returns Ok(data), where data is previous data evicted by this operation (if any).
    pub fn set(&mut self, entity: Entity, data: T) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            match self.vec.get_mut(entity.index as usize) {
                None => {
                    // Double capacity or grow enough to have room for the next index, if doubling is not enough
                    let new_len = usize::max(self.vec.capacity() * 2, entity.index as usize + 1);
                    self.vec.resize_with(new_len, || None);

                    self.vec[entity.index as usize] = Some(data);
                    Ok(None)
                }
                Some(entry) => Ok(entry.replace(data)),
            }
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
        if let Some(entry) = self.vec.get_mut(entity.index as usize) {
            entry.take()
        } else {
            None
        }
    }

    /// Remove the component for the given entity.
    /// Returns the previous data associated with the given entity in self.
    pub fn remove(&mut self, entity: Entity) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(entity) {
            if let Some(entry) = self.vec.get_mut(entity.index as usize) {
                Ok(entry.take())
            } else {
                Ok(None)
            }
        } else {
            Err(NoSuchEntity)
        }
    }

    /// Remove the data stored in self for all entities.
    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct VecTestData(i32);

    #[test]
    fn vec_get_not_set() {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);
        let entity = Entity {
            index: 0,
            generation: 0,
        };
        let entry = vec.get(entity);
        assert_eq!(entry, None);
    }

    #[test]
    fn vec_get() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(entity, data)?;
        assert_eq!(old_data, None);
        let entry = vec.get(entity);
        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn vec_set_exists() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(entity, data)?;
        assert_eq!(old_data, None);
        assert_eq!(vec.get(entity), Some(&data));

        let wrong_entity = Entity {
            index: 0,
            generation: 1,
        };
        assert!(vec.set(wrong_entity, VecTestData(69)).is_err()); //set with wrong entity
        Ok(())
    }

    #[test]
    fn can_insert_after_remove() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };

        let data = VecTestData(42);
        vec.set(entity, data)?;
        let removed_data = vec.remove(entity)?;
        assert_eq!(removed_data, Some(VecTestData(42)));

        let missing_entry = vec.get(entity);
        assert_eq!(missing_entry, None);

        let missing_entry = vec.set(entity, data)?;
        assert_eq!(missing_entry, None);

        let entry = vec.get(entity);
        assert_eq!(entry, Some(&VecTestData(42)));
        Ok(())
    }

    #[test]
    fn cannot_access_out_of_bounds() {
        let n = 3;
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);
        let entity = Entity {
            index: n,
            generation: 0,
        };
        let nope = vec.get(entity);
        assert_eq!(nope, None);
    }

    #[test]
    fn inserting_grows_vec_enough() -> Result<(), NoSuchEntity> {
        let capacity = 1;
        let n = 3;
        let entities = Arc::new(RwLock::new(Entities::new(capacity)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), capacity);
        let entity = {
            let mut lock = entities.write().unwrap();
            for _i in 0..n - 1 {
                lock.spawn();
            }
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(entity, data)?;
        assert_eq!(old_data, None);

        let value = vec.get(entity);
        assert_eq!(value, Some(&data));
        Ok(())
    }

    #[test]
    fn remove_missing_is_ok() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);
        let entity = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let remove_result = vec.remove(entity)?;
        assert_eq!(remove_result, None);
        Ok(())
    }

    #[test]
    fn test_iter_update() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let (entity1, entity2) = {
            let mut write = entities.write().unwrap();
            let entity1 = write.spawn();
            let entity2 = write.spawn();
            write.spawn(); // unused entity
            (entity1, entity2)
        };

        vec.set(entity1, VecTestData(1))?;
        vec.set(entity2, VecTestData(2))?;

        {
            let read = entities.read().unwrap();
            let mut expected_value = 1;

            for entity in read.iter() {
                if let Some(data) = vec.get_mut(entity) {
                    assert_eq!(data.0, expected_value);
                    expected_value += 1;
                    *data = VecTestData(40 + data.0);
                }
            }

            let mut expected_value = 41;
            for entity in read.iter() {
                if let Some(data) = vec.get(entity) {
                    assert_eq!(data.0, expected_value);
                    expected_value += 1;
                }
            }
        }
        Ok(())
    }
}
