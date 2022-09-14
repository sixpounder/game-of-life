use gtk::glib;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, glib::Enum, Copy, PartialEq)]
#[enum_type(name = "UniverseGridMode")]
pub enum UniverseGridMode {
    Design = 0,
    Run = 1,
}

impl Default for UniverseGridMode {
    fn default() -> Self {
        Self::Design
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniverseCell {
    Dead = 0,
    Alive = 1,
}

impl UniverseCell {
    pub fn is_alive(&self) -> bool {
        match self {
            UniverseCell::Alive => true,
            _ => false,
        }
    }
}

impl std::ops::Not for UniverseCell {
    type Output = UniverseCell;
    fn not(self) -> Self::Output {
        match self {
            UniverseCell::Alive => UniverseCell::Dead,
            UniverseCell::Dead => UniverseCell::Alive
        }
    }
}

impl fmt::Display for UniverseCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.is_alive() {
                true => "Alive",
                _ => "Dead",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct UniversePoint {
    row: usize,
    column: usize,
    cell: UniverseCell,
}

impl UniversePoint {
    pub fn new(row: usize, column: usize, cell: UniverseCell) -> Self {
        Self { row, column, cell }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn cell(&self) -> &UniverseCell {
        &self.cell
    }

    pub fn set_cell(&mut self, value: UniverseCell) {
        self.cell = value;
    }
}

pub trait UniversePointMatrix {
    type SetCellError;

    /// Gets the number of columns for this universe
    fn columns(&self) -> usize;

    /// Gets the number of rows for this universe
    fn rows(&self) -> usize;

    /// Gets a point at `row` and `column`
    fn get(&self, row: usize, column: usize) -> UniversePoint;

    /// Sets the cell state at `row` and `column` and, if successfull,
    /// returns the the altered point
    fn set(
        &mut self,
        row: usize,
        column: usize,
        value: UniverseCell,
    ) -> Result<UniversePoint, Self::SetCellError>;
}


