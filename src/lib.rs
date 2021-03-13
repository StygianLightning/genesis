mod entity;
mod mapstorage;
mod no_such_entity;
mod register;
mod vecstorage;

pub use entity::Entities;
pub use entity::Entity;
pub use mapstorage::MapStorage;
pub use no_such_entity::NoSuchEntity;
pub use register::Register;
pub use vecstorage::VecStorage;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct VecTestData(i32);

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct MapTestData(i32);

    struct World {
        entities: Arc<RwLock<Entities>>,
        pub vec: VecStorage<VecTestData>,
        pub map: MapStorage<MapTestData>,
    }

    impl World {
        pub fn new(capacity: u32) -> Self {
            let entities = Arc::new(RwLock::new(Entities::new(capacity)));
            let vec = VecStorage::new(Arc::clone(&entities), capacity);
            let map = MapStorage::new(Arc::clone(&entities));
            Self { entities, vec, map }
        }

        pub fn spawn(&mut self) -> Entity {
            self.entities.write().unwrap().spawn()
        }

        #[allow(unused)]
        pub fn despawn(&mut self, id: Entity) -> Result<(), NoSuchEntity> {
            let mut write = self.entities.write().unwrap();

            write.despawn(id)?;
            self.vec.remove_unchecked(id);
            self.map.remove_unchecked(id);

            Ok(())
        }
    }

    #[test]
    fn test_world() {
        let mut world = World::new(10);
        let first_id = world.spawn();
        let second_id = world.spawn();

        world.vec.set(first_id, VecTestData(1)).unwrap();
        world.vec.set(second_id, VecTestData(2)).unwrap();

        assert_eq!(world.vec.get(first_id), Some(&VecTestData(1)));

        let mut inc = 0;
        for id in world.entities.read().unwrap().iter() {
            inc += 1;
            assert_eq!(world.vec.get(id), Some(&VecTestData(inc)));
        }

        for id in world.entities.read().unwrap().iter() {
            world.vec.remove(id).unwrap();
        }
    }
}
