use crate::vec2::*;

/// Simple 2d matrix type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Grid<T: Default + Clone> {
    tiles: Vec<T>,
    pub size: IVec2,
}

impl<T: Default + Clone> Grid<T> {
    pub fn fill(size: IVec2, value: T) -> Grid<T> {
        Self {
            tiles: vec![value.clone(); (size.x * size.y) as usize],
            size,
        }
    }

    pub fn default(size: IVec2) -> Grid<T> {
        Grid::fill(size, T::default())
    }

    /// Caution: not bounds checked, call contains first
    pub fn get_at(&self, pos: &IVec2) -> &T {
        &self.tiles[pos_to_index(&self.size, pos)]
    }

    /// Caution: not bounds checked, call contains first
    pub fn get_at_mut(&mut self, pos: &IVec2) -> &mut T {
        &mut self.tiles[pos_to_index(&self.size, pos)]
    }

    /// Caution: not bounds checked, call contains first
    pub fn set_at(&mut self, pos: &IVec2, tile: T) {
        self.tiles[pos_to_index(&self.size, pos)] = tile;
    }

    /// Bound check
    pub fn contains(&self, pos: &IVec2) -> bool {
        rect_contains(&self.size, pos)
    }

    /// Returns the distance=1 neighbors and the relative position from `pos`
    pub fn get_adjacent(&self, pos: &IVec2) -> Vec<(&IVec2, &T)> {
        const ADJACENT: [IVec2; 4] = [
            IVec2 { x: 1, y: 0 },
            IVec2 { x: 0, y: 1 },
            IVec2 { x: -1, y: 0 },
            IVec2 { x: 0, y: -1 },
        ];
        ADJACENT
            .iter()
            .filter_map(|dir| {
                let look = *dir + *pos;
                self.contains(&look).then(|| (dir, self.get_at(&look)))
            })
            .collect()
    }

    /// Returns the diagonal with distance=2 neighbor and the relative position from `pos`
    pub fn get_diagonal(&self, pos: &IVec2) -> Vec<(&IVec2, &T)> {
        const DIAGONAL: [IVec2; 4] = [
            IVec2 { x: 1, y: 1 },
            IVec2 { x: 1, y: -1 },
            IVec2 { x: -1, y: 1 },
            IVec2 { x: -1, y: -1 },
        ];
        DIAGONAL
            .iter()
            .filter_map(|dir| {
                let look = *dir + *pos;
                self.contains(&look).then(|| (dir, self.get_at(&look)))
            })
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (IVec2, &T)> {
        iter_area(self.size).map(|pos| (pos, self.get_at(&pos)))
    }
}

pub fn pos_to_index(size: &IVec2, pos: &IVec2) -> usize {
    (pos.x + pos.y * size.x) as usize
}

pub fn rect_contains(size: &IVec2, pos: &IVec2) -> bool {
    (0..size.x).contains(&pos.x) && (0..size.y).contains(&pos.y)
}

pub fn iter_area(size: IVec2) -> impl Iterator<Item = IVec2> {
    (0..size.y)
        .map(move |y| (0..size.x).map(move |x| IVec2::new(x, y)))
        .flatten()
}
