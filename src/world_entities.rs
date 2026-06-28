#[derive(Default, Debug, Clone, Copy)]
pub struct Food {
    pub pos_x: i32,
    pub pos_y: i32,
    pub amount: i32,
    pub is_harvested: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Obstacle {
    #[default]
    Normal,
    Border,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Terrain {
    #[default]
    Empty,
    Food(Food),
    Obstacle {
        kind: Obstacle,
    },
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellContents {
    pub terrain: Terrain,
    pub foraging_strength: f32,
    pub to_home_strength: f32,
}

impl CellContents {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain,
            ..Self::default()
        }
    }
}
