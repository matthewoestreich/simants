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
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PheromoneKind {
    #[default]
    ToHome,
    ToFood,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Pheromone {
    pub kind: PheromoneKind,
    pub strength: f32,
}

impl Pheromone {
    pub fn new(kind: PheromoneKind, strength: f32) -> Self {
        Self { kind, strength }
    }

    pub fn set_strength(&mut self, value: f32) {
        self.strength = value;
    }

    pub fn weaken(&mut self, amount: f32) {
        self.strength -= amount;
    }

    pub fn strengthen(&mut self, amount: f32) {
        self.strength += amount;
    }
}
