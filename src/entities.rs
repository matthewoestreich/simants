#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Food {
    pub pos_x: i32,
    pub pos_y: i32,
    pub amount: i32,
    pub is_harvested: bool,
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

#[derive(Default, Debug, Clone, Copy)]
pub enum PheromoneKind {
    #[default]
    ToHome,
    ToFood,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Pheromone {
    kind: PheromoneKind,
    strength: f32,
}

impl Pheromone {
    pub fn new(kind: PheromoneKind, strength: f32) -> Self {
        Self { kind, strength }
    }

    pub fn kind(&self) -> PheromoneKind {
        self.kind
    }

    pub fn strength(&self) -> f32 {
        self.strength
    }

    pub fn set_strength(&mut self, value: f32) {
        self.strength = value;
    }

    pub fn weaken(&mut self, amount: f32) {
        let new_strength = self.strength - amount;
        self.strength = new_strength.max(0.0);
    }

    pub fn strengthen(&mut self, amount: f32) {
        self.strength += amount;
    }
}
