use std::collections::{HashMap, HashSet};

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

#[derive(PartialEq)]
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
    dirty_tiles: HashSet<usize>,
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
            dirty_tiles: HashSet::new(),
        }
    }

    #[inline]
    pub fn tile_at_pos(&self, pos_x: u32, pos_y: u32) -> &Tile {
        &self.nodes[((pos_x as usize * self.len()) + pos_y as usize).clamp(0, self.len() - 1)]
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

    #[inline]
    pub fn mark_tile_dirty(&mut self, i: usize) {
        self.dirty_tiles.insert(i);
    }

    pub fn mark_rect_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let xmax = (x + w).min(self.width);
        let ymax = (y + h).min(self.height);

        for ty in y..ymax {
            let row = (ty * self.width) as usize;
            for tx in x..xmax {
                self.dirty_tiles.insert(row + tx as usize);
            }
        }
    }

    /// Take and clear the current dirty set (renderer calls this when camera is static).
    pub fn take_dirty_tiles(&mut self) -> Vec<usize> {
        if self.dirty_tiles.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(self.dirty_tiles.len());
        for i in self.dirty_tiles.drain() {
            out.push(i);
        }
        out
    }

    pub fn set_tile_visual(&mut self, x: u32, y: u32, visual: TileVisual) {
        let i = self.idx_xy(x, y);
        if self.nodes[i].visual != visual {
            self.nodes[i].visual = visual;
            self.mark_tile_dirty(i);
        }
    }

    pub fn set_tile_shape_id(&mut self, x: u32, y: u32, shape_id: u16) {
        let i = self.idx_xy(x, y);
        if self.nodes[i].visual.shape_id != shape_id {
            self.nodes[i].visual.shape_id = shape_id;
            self.mark_tile_dirty(i);
        }
    }
}
