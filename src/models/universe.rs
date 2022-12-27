use super::{UniverseCell, UniversePoint, UniversePointMatrix};
use crate::config::G_LOG_DOMAIN;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

const UNIVERSE_RANDOM_ALIVE_PROBABILITY: f64 = 0.6;
const UNIVERSE_CELL_INITIAL_CORPSE_HEAT: f64 = 0.65;
const UNIVERSE_DEFAULT_FREEZE_RATE: f64 = 0.30;

fn compute_initial_delta(universe: &mut Universe) {
    let mut initial_delta: Vec<UniversePoint> = vec![];
    for row in 0..universe.rows {
        for column in 0..universe.columns {
            let index = universe.get_index(row, column);
            let delta_point = UniversePoint::new(
                row,
                column,
                universe.cells[index],
                universe.death_map[index],
            );
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
    death_map: Vec<f64>,
    corpse_freeze_rate: f64,
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

    pub fn new_empty(rows: usize, columns: usize) -> Universe {
        Self::create(rows, columns)
    }

    fn create(rows: usize, columns: usize) -> Universe {
        let s: usize = (rows * columns) as usize;
        let mut cells: Vec<UniverseCell> = Vec::with_capacity(s);
        let mut death_map: Vec<f64> = Vec::with_capacity(s);

        for i in 0..s {
            cells.insert(i, UniverseCell::Dead);
            death_map.insert(i, 0.0);
        }

        let universe = Universe {
            rows,
            columns,
            cells,
            corpse_freeze_rate: UNIVERSE_DEFAULT_FREEZE_RATE,
            death_map,
            generations: 0,
            last_delta: None,
        };

        universe
    }

    /// Seeds this universe with random values
    fn random_seed(&mut self) {
        let mut rng = rand::thread_rng();
        for i in 0..self.rows {
            for j in 0..self.columns {
                let y: f64 = rng.gen();
                if y >= UNIVERSE_RANDOM_ALIVE_PROBABILITY {
                    self.set_cell(i, j, UniverseCell::Alive);
                } else {
                    self.set_cell(i, j, UniverseCell::Dead);
                }
            }
        }
    }

    #[allow(dead_code)]
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
    pub fn get_cell(&self, row: usize, column: usize) -> (&UniverseCell, &f64) {
        let idx = self.get_index(row, column);
        match self.cells.get(idx) {
            Some(cell) => (cell, self.death_map.get(idx).unwrap_or(&0.0)),
            None => panic!("Could not get cell at row {} column {}", row, column),
        }
    }

    /// Computes the next state of a cell given its current state
    /// and the state of its neighbours
    fn cell_next_state(&self, row: usize, column: usize) -> UniverseCell {
        let (cell, _) = self.get_cell(row, column);
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
            let cell_current_state = point.cell();
            let cell_next_state = self.cell_next_state(point.row(), point.column());
            let death_map_item_ref = new_state.death_map.get_mut(index).unwrap();
            new_state.cells[index] = cell_next_state;

            if cell_next_state != *cell_current_state {
                match cell_next_state {
                    UniverseCell::Alive => {
                        // Cell becomes alive
                        *death_map_item_ref = 0.0;
                    }
                    UniverseCell::Dead => {
                        // Cell dies
                        *death_map_item_ref = UNIVERSE_CELL_INITIAL_CORPSE_HEAT;
                    }
                }
                delta.push(UniversePoint::new(
                    point.row(),
                    point.column(),
                    cell_next_state,
                    *death_map_item_ref,
                ));
            } else {
                // Dead cell corpse keeps freezing
                if *cell_current_state == UniverseCell::Dead && *death_map_item_ref > 0.0 {
                    *death_map_item_ref = point.corpse_heat() - UNIVERSE_DEFAULT_FREEZE_RATE;
                }
            }
        }
        self.cells = new_state.cells.clone();
        self.death_map = new_state.death_map.clone();
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

    pub fn corpse_freeze_rate(&self) -> &f64 {
        &self.corpse_freeze_rate
    }

    pub fn set_corpse_freeze_rate(&mut self, value: f64) {
        self.corpse_freeze_rate = value;
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

    fn get(&self, row: usize, column: usize) -> Option<UniversePoint> {
        let idx = self.get_index(row, column);
        match self.cells.get(idx) {
            Some(cell) => Some(UniversePoint::new(row, column, *cell, *self.death_map.get(idx).unwrap_or(&0.0))),
            None => None,
        }
    }

    fn set(
        &mut self,
        row: usize,
        column: usize,
        value: UniverseCell,
    ) -> Result<UniversePoint, Self::SetCellError> {
        self.set_cell(row, column, value);
        Ok(self.get(row, column).unwrap())
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
            let (last, last_corpse_heath) = self.universe.get_cell(self.row, self.column);
            self.row += 1;
            self.column += 1;

            return Some(UniversePoint::new(
                self.row - 1,
                self.column - 1,
                *last,
                *last_corpse_heath,
            ));
        }

        let (cell, corpse_heat) = self.universe.get_cell(self.row, self.column);

        let point = UniversePoint::new(self.row, self.column, *cell, *corpse_heat);

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

#[derive(Serialize, Deserialize, Debug)]
pub struct UniverseSnapshot {
    rows: usize,
    columns: usize,
    cells: Vec<UniverseCell>,

    #[serde(skip, default)]
    death_map: Vec<f64>,
}

impl From<&Universe> for UniverseSnapshot {
    fn from(value: &Universe) -> Self {
        UniverseSnapshot {
            cells: value.cells.clone(),
            death_map: value.death_map.clone(),
            rows: value.rows(),
            columns: value.columns(),
        }
    }
}

impl UniverseSnapshot {
    fn get_index(&self, row: usize, column: usize) -> usize {
        ((row * self.columns) + column) as usize
    }

    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
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

    fn get(&self, row: usize, column: usize) -> Option<UniversePoint> {
        let idx = self.get_index(row, column);
        match self.cells.get(idx) {
            Some(cell) => Some(UniversePoint::new(row, column, *cell, self.death_map[idx])),
            None => None,
        }
    }

    fn set(
        &mut self,
        _row: usize,
        _column: usize,
        _value: UniverseCell,
    ) -> Result<UniversePoint, Self::SetCellError> {
        Err("UniverseSnapshot is readonly")
    }
}

#[derive(Debug)]
pub enum SnapshotError {
    Invalid,
}

impl<'a> TryFrom<&Vec<u8>> for UniverseSnapshot {
    type Error = SnapshotError;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        match bincode::deserialize::<Self>(value.as_ref()) {
            Ok(snapshot) => Ok(snapshot),
            Err(error) => {
                glib::g_critical!(G_LOG_DOMAIN, "{}", error);
                Err(SnapshotError::Invalid)
            }
        }
    }
}

impl From<UniverseSnapshot> for Universe {
    fn from(snapshot: UniverseSnapshot) -> Self {
        let mut death_map = Vec::with_capacity(snapshot.rows * snapshot.columns);
        death_map.fill(0.0);

        Self {
            rows: snapshot.rows,
            columns: snapshot.columns,
            corpse_freeze_rate: UNIVERSE_DEFAULT_FREEZE_RATE,
            death_map,
            cells: snapshot.cells.clone(),
            generations: 0,
            last_delta: None,
        }
    }
}
