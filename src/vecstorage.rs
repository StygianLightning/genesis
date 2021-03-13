use super::entity::Entity;
use crate::no_such_entity::NoSuchEntity;
use crate::Entities;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct VecStorage<T> {
    vec: Vec<Option<T>>,
    entities: Arc<RwLock<Entities>>,
}

impl<T> VecStorage<T> {
    pub fn new(entities: Arc<RwLock<Entities>>, capacity: u32) -> Self {
        let mut vec = vec![];
        vec.resize_with(capacity as usize, Default::default);
        Self { vec, entities }
    }

    pub fn get(&self, id: Entity) -> Option<&T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            self.vec.get(id.index as usize).unwrap_or(&None).as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Entity) -> Option<&mut T> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            if let Some(entry) = self.vec.get_mut(id.index as usize) {
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
            match self.vec.get_mut(id.index as usize) {
                None => {
                    // Double capacity or grow enough to have room for the next index, if doubling is not enough
                    let new_len = usize::max(self.vec.capacity() * 2, id.index as usize + 1);
                    self.vec.resize_with(new_len, || None);

                    self.vec[id.index as usize] = Some(data);
                    Ok(None)
                }
                Some(entry) => Ok(entry.replace(data)),
            }
        } else {
            Err(NoSuchEntity {})
        }
    }

    pub fn remove_unchecked(&mut self, id: Entity) -> Option<T> {
        if let Some(entry) = self.vec.get_mut(id.index as usize) {
            entry.take()
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: Entity) -> Result<Option<T>, NoSuchEntity> {
        let lock = self.entities.read().unwrap();
        if lock.exists(id) {
            if let Some(entry) = self.vec.get_mut(id.index as usize) {
                Ok(entry.take())
            } else {
                Ok(None)
            }
        } else {
            Err(NoSuchEntity)
        }
    }

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
        let id = Entity {
            index: 0,
            generation: 0,
        };
        let entry = vec.get(id);
        assert_eq!(entry, None);
    }

    #[test]
    fn vec_get() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(id, data)?;
        assert_eq!(old_data, None);
        let entry = vec.get(id);
        assert_eq!(entry, Some(&data));
        Ok(())
    }

    #[test]
    fn vec_set_exists() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(id, data)?;
        assert_eq!(old_data, None);
        assert_eq!(vec.get(id), Some(&data));

        let wrong_id = Entity {
            index: 0,
            generation: 1,
        };
        assert!(vec.set(wrong_id, VecTestData(69)).is_err()); //set with wrong ID
        Ok(())
    }

    #[test]
    fn can_insert_after_remove() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };

        let data = VecTestData(42);
        vec.set(id, data)?;
        let removed_data = vec.remove(id)?;
        assert_eq!(removed_data, Some(VecTestData(42)));

        let missing_entry = vec.get(id);
        assert_eq!(missing_entry, None);

        let missing_entry = vec.set(id, data)?;
        assert_eq!(missing_entry, None);

        let entry = vec.get(id);
        assert_eq!(entry, Some(&VecTestData(42)));
        Ok(())
    }

    #[test]
    fn cannot_access_out_of_bounds() {
        let n = 3;
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);
        let id = Entity {
            index: n,
            generation: 0,
        };
        let nope = vec.get(id);
        assert_eq!(nope, None);
    }

    #[test]
    fn inserting_grows_vec_enough() -> Result<(), NoSuchEntity> {
        let capacity = 1;
        let n = 3;
        let entities = Arc::new(RwLock::new(Entities::new(capacity)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), capacity);
        let id = {
            let mut lock = entities.write().unwrap();
            for _i in 0..n - 1 {
                lock.spawn();
            }
            lock.spawn()
        };
        let data = VecTestData(42);
        let old_data = vec.set(id, data)?;
        assert_eq!(old_data, None);

        let value = vec.get(id);
        assert_eq!(value, Some(&data));
        Ok(())
    }

    #[test]
    fn remove_missing_is_ok() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);
        let id = {
            let mut lock = entities.write().unwrap();
            lock.spawn()
        };
        let remove_result = vec.remove(id)?;
        assert_eq!(remove_result, None);
        Ok(())
    }

    #[test]
    fn test_iter_update() -> Result<(), NoSuchEntity> {
        let entities = Arc::new(RwLock::new(Entities::new(3)));
        let mut vec = VecStorage::<VecTestData>::new(Arc::clone(&entities), 3);

        let (id1, id2) = {
            let mut write = entities.write().unwrap();
            let id1 = write.spawn();
            let id2 = write.spawn();
            write.spawn(); // unused id
            (id1, id2)
        };

        vec.set(id1, VecTestData(1))?;
        vec.set(id2, VecTestData(2))?;

        {
            let read = entities.read().unwrap();
            let mut expected_value = 1;

            for id in read.iter() {
                if let Some(data) = vec.get_mut(id) {
                    assert_eq!(data.0, expected_value);
                    expected_value += 1;
                    *data = VecTestData(40 + data.0);
                }
            }

            let mut expected_value = 41;
            for id in read.iter() {
                if let Some(data) = vec.get(id) {
                    assert_eq!(data.0, expected_value);
                    expected_value += 1;
                }
            }
        }
        Ok(())
    }
}
