use bevy::{math::Vec3, utils::HashMap};

const NEIGHBOR_OFFSETS: [(i32, i32); 9] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 0),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

pub trait Position {
    fn position(&self) -> Vec3;
}

/// A 2D spatial grid that stores entities with a position.
///
/// The grid is divided into cells of a fixed size, and each cell stores a list
/// of entities that are within that cell. This allows for fast queries of
/// entities within a certain radius.
#[derive(Debug, Clone)]
pub struct SpatialGrid2D<T: Position> {
    radius: f32,
    inner: HashMap<(i32, i32), Vec<T>>,
}

impl<T: Position> SpatialGrid2D<T> {
    /// Creates a new grid supporting queries with the given radius.
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            inner: HashMap::new(),
        }
    }

    /// Inserts an entity into the grid.
    pub fn insert(&mut self, entity: T) {
        self.inner
            .entry(self.get_key(entity.position()))
            .or_insert_with(Vec::new)
            .push(entity);
    }

    /// Retrieves entities within the radius of the given position.
    pub fn query(&self, position: Vec3) -> Vec<&T> {
        let key = self.get_key(position);

        NEIGHBOR_OFFSETS
            .iter()
            .map(|offset| (key.0 + offset.0, key.1 + offset.1))
            .filter_map(|key| self.inner.get(&key))
            .flatten()
            .filter(|entity| (entity.position() - position).length() <= self.radius)
            .collect()
    }

    /// Clears the grid.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Recomputes the grid after entities have been moved.
    pub fn recompute(&mut self) {
        let old_inner = std::mem::take(&mut self.inner);
        for entity in old_inner.into_values().flatten() {
            self.insert(entity);
        }
    }

    /// Returns an iterator over all entities in the grid.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.values().flatten()
    }

    /// Returns a mutable iterator over all entities in the grid.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.values_mut().flatten()
    }

    fn get_key(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x / self.radius).floor() as i32,
            (position.y / self.radius).floor() as i32,
        )
    }
}
