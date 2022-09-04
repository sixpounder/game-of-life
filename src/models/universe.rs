use super::{UniverseCell, UniversePoint, UniversePointMatrix};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

fn compute_initial_delta(universe: &mut Universe) {
    let mut initial_delta: Vec<UniversePoint> = vec![];
    for row in 0..universe.rows {
        for column in 0..universe.columns {
            let index = universe.get_index(row, column);
            let delta_point = UniversePoint::new(row, column, universe.cells[index]);
            initial_delta.push(delta_point);
        }
    }

    universe.last_delta = Some(initial_delta);
}

/// Represents a universe as a collection of "cells"
/// which can be in two states: `Alive` or `Dead`
#[derive(Clone, Debug)]
pub struct Universe {
    columns: usize,
    rows: usize,
    cells: Vec<UniverseCell>,
    generations: u64,
    last_delta: Option<Vec<UniversePoint>>,
}

impl Default for Universe {
    fn default() -> Self {
        Universe::new_random(200, 200)
    }
}

impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        let mut universe = Self::create(width, height);
        compute_initial_delta(&mut universe);
        universe
    }

    pub fn new_random(rows: usize, columns: usize) -> Universe {
        let mut universe = Self::create(rows, columns);
        universe.random_seed();
        compute_initial_delta(&mut universe);
        universe
    }

    fn create(rows: usize, columns: usize) -> Universe {
        let s: usize = (rows * columns) as usize;
        let mut cells: Vec<UniverseCell> = Vec::with_capacity(s);

        for i in 0..s {
            cells.insert(i, UniverseCell::Dead);
        }

        let universe = Universe {
            rows: rows,
            columns: columns,
            cells,
            generations: 0,
            last_delta: None,
        };

        universe
    }

    /// Seeds this universe with random values
    fn random_seed(&mut self) {
        let mut rng = rand::thread_rng();
        for i in 0..self.rows - 1 {
            for j in 0..self.columns - 1 {
                let y: f64 = rng.gen();
                if y >= 0.5 {
                    self.set_cell(i, j, UniverseCell::Alive);
                } else {
                    self.set_cell(i, j, UniverseCell::Dead);
                }
            }
        }
    }

    fn clear_delta(&mut self) {
        self.last_delta = None;
    }

    fn get_index(&self, row: usize, column: usize) -> usize {
        ((row * self.columns) + column) as usize
    }

    /// Sets cell at `row`x`column` coordinates
    pub fn set_cell(&mut self, row: usize, column: usize, cell: UniverseCell) {
        let i = self.get_index(row, column);
        self.cells[i] = cell;
    }

    /// Gets the cell at `row`x`column`.
    /// # Panics
    /// Panics if no cell is found
    pub fn get_cell(&self, row: usize, column: usize) -> &UniverseCell {
        match self.cells.get(self.get_index(row, column)) {
            Some(cell) => cell,
            None => panic!("Could not get cell at row {} column {}", row, column),
        }
    }

    /// Computes the next state of a cell given its current state
    /// and the state of its neighbours
    fn cell_next_state(&self, row: usize, column: usize) -> UniverseCell {
        let cell = self.get_cell(row, column);
        let alive_cells_around = self.cell_living_neighbours_count(row, column);

        match cell {
            UniverseCell::Alive => {
                if alive_cells_around < 2 || alive_cells_around > 3 {
                    UniverseCell::Dead
                } else if alive_cells_around == 2 || alive_cells_around == 3 {
                    UniverseCell::Alive
                } else {
                    UniverseCell::Alive
                }
            }

            UniverseCell::Dead => {
                if alive_cells_around == 3 {
                    UniverseCell::Alive
                } else {
                    UniverseCell::Dead
                }
            }
        }
    }

    /// Counts living adiacents cells for a given cell at `row`x`column` coordinates
    fn cell_living_neighbours_count(&self, row: usize, column: usize) -> u8 {
        let mut count = 0;
        let rows = [self.rows - 1, 0, 1];
        let cols = [self.columns - 1, 0, 1];
        for delta_row in rows.iter() {
            for delta_col in cols.iter() {
                if *delta_row == 0 && *delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.rows;
                let neighbor_col = (column + delta_col) % self.columns;
                let idx = self.get_index(neighbor_row, neighbor_col);
                match self.cells[idx] {
                    UniverseCell::Alive => count += 1,
                    UniverseCell::Dead => (),
                };
            }
        }
        count
    }

    /// Iterates over this universe and computes its next generation.
    /// Alters the struct in-place.
    pub fn tick(&mut self) {
        let mut new_state = Self::new(self.columns, self.rows);
        let mut delta: Vec<UniversePoint> = Vec::with_capacity(self.cells.capacity());
        for point in self.iter_cells() {
            let index = self.get_index(point.row(), point.column());
            new_state.cells[index] = self.cell_next_state(point.row(), point.column());

            if new_state.cells[index] != self.cells[index] {
                delta.push(UniversePoint::new(
                    point.row(),
                    point.column(),
                    new_state.cells[index],
                ));
            }
        }
        self.cells = new_state.cells.clone();
        drop(new_state);
        self.generations += 1;
        self.last_delta = Some(delta);
    }

    /// Counts and returns the number of alive cells
    /// in this universe
    pub fn alive_cells_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|cell| match cell {
                UniverseCell::Alive => true,
                UniverseCell::Dead => false,
            })
            .count()
    }

    /// Counts and returns the number of dead cells
    /// in this universe
    pub fn dead_cells_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|cell| match cell {
                UniverseCell::Alive => false,
                UniverseCell::Dead => true,
            })
            .count()
    }

    /// Gets the last delta for this universe.
    /// If no iterations have been performed, this will be a delta
    /// with the current values of all the universe itself
    pub fn last_delta(&self) -> Vec<UniversePoint> {
        match &self.last_delta {
            Some(delta) => delta.to_vec(),
            None => self.iter_cells().collect::<Vec<UniversePoint>>(),
        }
    }

    pub fn iter_cells(&self) -> UniverseIterator {
        UniverseIterator::new(&self)
    }

    pub fn snapshot(&self) -> UniverseSnapshot {
        UniverseSnapshot::from(self)
    }
}

impl UniversePointMatrix for Universe {
    type SetCellError = ();

    fn rows(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.columns
    }

    fn get(&self, row: usize, column: usize) -> UniversePoint {
        match self.cells.get(self.get_index(row, column)) {
            Some(cell) => UniversePoint::new(row, column, *cell),
            None => panic!("Could not get cell at row {} column {}", row, column),
        }
    }

    fn set(
        &mut self,
        row: usize,
        column: usize,
        value: UniverseCell,
    ) -> Result<UniversePoint, Self::SetCellError> {
        self.set_cell(row, column, value);
        Ok(self.get(row, column))
    }
}

pub struct UniverseIterator<'a> {
    universe: &'a Universe,
    row: usize,
    column: usize,
}

impl<'a> UniverseIterator<'a> {
    pub fn new(universe: &'a Universe) -> Self {
        Self {
            universe,
            row: 0,
            column: 0,
        }
    }
}

impl<'a> Iterator for UniverseIterator<'a> {
    type Item = UniversePoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row == self.universe.rows() && self.column == self.universe.columns() {
            return None;
        }

        if self.row == self.universe.rows() - 1 && self.column == self.universe.columns() - 1 {
            // This is the last item. Return it and set counters to above the last item
            let last = self.universe.get_cell(self.row, self.column);
            self.row += 1;
            self.column += 1;

            return Some(UniversePoint::new(self.row - 1, self.column - 1, *last));
        }

        let point = UniversePoint::new(
            self.row,
            self.column,
            *self.universe.get_cell(self.row, self.column),
        );

        if self.column == self.universe.columns() - 1 {
            self.column = 0;
            self.row += 1;
        } else {
            self.column += 1;
        }

        return Some(point);
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.columns()) {
            for &cell in line {
                let symbol = if cell == UniverseCell::Dead {
                    '◻'
                } else {
                    '◼'
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")
    }
}

impl Drop for Universe {
    fn drop(&mut self) {
        self.cells.clear();
        self.generations = 0;
    }
}

pub enum SnapshotError {}

#[derive(Serialize, Deserialize, Debug)]
pub struct UniverseSnapshot {
    columns: usize,
    rows: usize,
    cells: Vec<UniverseCell>,
}

impl From<&Universe> for UniverseSnapshot {
    fn from(value: &Universe) -> Self {
        UniverseSnapshot {
            cells: value.cells.clone(),
            rows: value.rows(),
            columns: value.columns(),
        }
    }
}

impl UniverseSnapshot {
    fn get_index(&self, row: usize, column: usize) -> usize {
        ((row * self.columns) + column) as usize
    }
}

impl UniversePointMatrix for UniverseSnapshot {
    type SetCellError = &'static str;

    fn rows(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.columns
    }

    fn get(&self, row: usize, column: usize) -> UniversePoint {
        match self.cells.get(self.get_index(row, column)) {
            Some(cell) => UniversePoint::new(row, column, *cell),
            None => panic!("Could not get cell at row {} column {}", row, column),
        }
    }

    fn set(
        &mut self,
        row: usize,
        column: usize,
        value: UniverseCell,
    ) -> Result<UniversePoint, Self::SetCellError> {
        Err("UniverseSnapshot is readonly")
    }
}

impl From<UniverseSnapshot> for Universe {
    fn from(snapshot: UniverseSnapshot) -> Self {
        Self {
            rows: snapshot.rows,
            columns: snapshot.columns,
            cells: snapshot.cells.clone(),
            generations: 0,
            last_delta: None,
        }
    }
}


