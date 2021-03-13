use crate::no_such_entity::NoSuchEntity;
use serde::{Deserialize, Serialize};

/// An entity.
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum EntityIDEntry {
    Used(u32),
    Unused(u32),
}

impl Default for EntityIDEntry {
    fn default() -> Self {
        EntityIDEntry::Unused(0)
    }
}

impl EntityIDEntry {
    pub fn is_unused(&self) -> bool {
        match self {
            EntityIDEntry::Unused(_) => true,
            _ => false,
        }
    }
}

/// A collection of entities.
#[derive(Debug)]
pub struct Entities {
    ids: Vec<EntityIDEntry>,
}

impl Entities {
    /// Allocate a set of entities with the given initial capacity.
    pub fn new(capacity: u32) -> Self {
        let mut vec = vec![];
        vec.resize(capacity as usize, EntityIDEntry::Unused(0));
        Self { ids: vec }
    }

    /// Spawn a new entity. This will grow the collection if necessary.
    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.ids.iter().position(|id| id.is_unused()) {
            match self.ids[index] {
                EntityIDEntry::Unused(gen) => {
                    let entity_id = Entity {
                        generation: gen,
                        index: index as u32,
                    };
                    self.ids[index] = EntityIDEntry::Used(gen);
                    entity_id
                }
                _ => unreachable!(),
            }
        } else {
            let next_idx = self.ids.len() as u32;
            let gen = 0;
            let entity_id = Entity {
                index: next_idx,
                generation: gen,
            };
            self.ids.push(EntityIDEntry::Used(gen));
            entity_id
        }
    }

    /// Iterate over all existing entities.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.ids
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| match entry {
                EntityIDEntry::Used(gen) => Some(Entity {
                    index: i as u32,
                    generation: *gen,
                }),
                _ => None,
            })
    }

    /// Check if an entity exists.
    pub fn exists(&self, id: Entity) -> bool {
        if let Some(entry) = self.ids.get(id.index as usize) {
            match entry {
                EntityIDEntry::Unused(_) => false,
                EntityIDEntry::Used(generation) => *generation == id.generation,
            }
        } else {
            false
        }
    }

    #[doc(hidden)]
    pub fn despawn(&mut self, id: Entity) -> Result<(), NoSuchEntity> {
        if let Some(EntityIDEntry::Used(generation)) = self.ids.get(id.index as usize) {
            if id.generation == *generation {
                self.ids[id.index as usize] = EntityIDEntry::Unused(generation.wrapping_add(1));
                return Ok(());
            }
        }
        Err(NoSuchEntity)
    }

    /// Remove all entities.
    pub fn clear(&mut self) {
        for id in &mut self.ids {
            if let EntityIDEntry::Used(generation) = id {
                *id = EntityIDEntry::Unused(generation.wrapping_add(1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id() {
        let mut id_allocator = Entities::new(3);
        let next_id = id_allocator.spawn();
        let first_id = Entity {
            index: 0,
            generation: 0,
        };
        assert_eq!(next_id, first_id);
        assert!(id_allocator.exists(next_id));
        assert_eq!(
            id_allocator.ids[0],
            EntityIDEntry::Used(first_id.generation)
        );
    }

    #[test]
    fn hierarchy_grows() {
        let mut id_allocator = Entities::new(0);
        assert_eq!(id_allocator.ids.len(), 0);
        let first_id = Entity {
            index: 0,
            generation: 0,
        };
        let next_id = id_allocator.spawn();
        assert_eq!(next_id, first_id);
        assert!(id_allocator.exists(next_id));
        assert_eq!(
            id_allocator.ids[0],
            EntityIDEntry::Used(first_id.generation)
        );
        assert!(!id_allocator.ids.is_empty());
    }

    #[test]
    fn remove_id_grows_generation() -> Result<(), NoSuchEntity> {
        //even when switching to not getting the first free index in the vec,
        //size 1 guarantees that index 0 will be re-used in this test.
        let mut id_allocator = Entities::new(1);
        let next_id = id_allocator.spawn();
        id_allocator.despawn(next_id)?;
        assert_eq!(id_allocator.ids[0], EntityIDEntry::Unused(1));
        let second_id = Entity {
            index: 0,
            generation: 1,
        };
        let next_id = id_allocator.spawn();
        assert_eq!(next_id, second_id);
        Ok(())
    }
}
