use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Food {
    pub amount: f32,
}

impl Default for Food {
    fn default() -> Self {
        Self { amount: FOOD_MAX }
    }
}

impl Food {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Terrain {
    #[default]
    Empty,
    Food,
    Colony,
    Obstacle,
    Border,
    Invalid,
}

impl Terrain {
    // We are obstructed by obstacles and borders
    pub fn is_obstruction(&self) -> bool {
        matches!(self, Terrain::Obstacle | Terrain::Border | Terrain::Invalid)
    }

    pub fn is_food(&self) -> bool {
        matches!(self, Terrain::Food)
    }

    pub fn is_colony(&self) -> bool {
        matches!(self, Terrain::Colony)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, Terrain::Invalid)
    }
}
