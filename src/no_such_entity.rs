use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Error indicating that an entity passed to some operation doesn't exist.
/// This usually indicates that the generational index of the entity was outdated.
#[derive(Error, Debug)]
pub struct NoSuchEntity;

impl Display for NoSuchEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No such entity")
    }
}
