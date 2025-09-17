use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainKind {
    Unknown,
    Grass,
    Rock,
    Sand,
    Water,
    Road,
    Mud,
}

pub struct TileVisual {
    pub shape_id: u16,
}

pub type CostMod = u8;

pub struct Tile {
    pub terrain: TerrainKind,
    pub passable: bool,
    pub buildable: bool,
    pub occupied: bool,
    pub cost_mod: CostMod,
    pub visual: TileVisual,
}

struct Reservation {
    id: u64,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    ttl: u32,
}

pub type MapNode = Tile;

pub struct Map {
    pub width: u32,
    pub height: u32,
    nodes: Vec<Tile>,
    reservations: HashMap<u64, Reservation>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width * height) as usize;
        Self {
            width,
            height,
            nodes: (0..len)
                .map(|_| Tile {
                    terrain: TerrainKind::Unknown,
                    passable: true,
                    buildable: true,
                    occupied: false,
                    cost_mod: 0,
                    visual: TileVisual { shape_id: 0 },
                })
                .collect(),
            reservations: HashMap::new(),
        }
    }

    #[inline]
    pub fn tile_at_index(&self, i: usize) -> &Tile {
        &self.nodes[i]
    }
    #[inline]
    pub fn tile_at_index_mut(&mut self, i: usize) -> &mut Tile {
        &mut self.nodes[i]
    }
    #[inline]
    pub fn idx_xy(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}
