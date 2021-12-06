use core::convert::{TryFrom, TryInto};

#[derive(Debug, Copy, Clone)]
pub enum StoneColor {
    White,
    Black,
}

impl StoneColor {
    pub fn name(&self) -> &str {
        match self {
            Self::White => "white",
            Self::Black => "black",
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stone {
    pub color: StoneColor,
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct OptCoords {
    pub row: Option<u8>,
    pub col: Option<u8>,
}

impl Default for OptCoords {
    fn default() -> Self {
        Self {
            row: None,
            col: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Coords {
    pub row: u8,
    pub col: u8,
}

impl Coords {
    pub fn from(row: u8, col: u8) -> Self {
        Self { row, col }
    }

    pub fn vertex(&self) -> (i32, i32) {
        (self.col as i32, self.row as i32)
    }
}

impl TryFrom<&OptCoords> for Coords {
    type Error = ();

    fn try_from(opt_coords: &OptCoords) -> Result<Coords, ()> {
        match opt_coords {
            OptCoords {
                col: Some(col),
                row: Some(row),
            } => Ok(Coords::from(row.clone(), col.clone())),
            _ => Err(()),
        }
    }
}
