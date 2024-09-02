pub struct MapNode {}

pub struct Map {
    nodes: Vec<MapNode>,
    width: u32,
    height: u32,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            nodes: Self::init_nodes(width * height),
        }
    }

    fn init_nodes(count: u32) -> Vec<MapNode> {
        let mut nodes = vec![];
        for i in 0..count {
            nodes.push(MapNode {});
        }
        nodes
    }

    pub fn get_node(&self, x: u32, y: u32) -> &MapNode {
        &self.nodes[x as usize * y as usize + y as usize]
    }
}
