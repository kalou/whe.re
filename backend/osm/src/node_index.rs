use std::collections::HashMap;

use graph;
use quadtree;
use radix_trie;
use radix_trie::TrieCommon;

use super::Node;

#[derive(Debug)]
pub struct NodePos {
    lat: f64,
    lon: f64,
    id: i64
}

impl quadtree::Point for NodePos
{
    fn get_x(&self) -> f64 { self.lon }
    fn get_y(&self) -> f64 { self.lat }
}

// This is mapping to node ids
#[derive(Debug)]
pub struct NodeIndex {
    by_poi: HashMap<String, Vec<i64>>,
    by_name: radix_trie::Trie<String, Vec<i64>>,
    by_pos: quadtree::QuadTree<NodePos>,
}

impl NodeIndex {
    pub fn new(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        NodeIndex {
            by_poi: HashMap::new(),
            by_name: radix_trie::Trie::new(),
            by_pos: quadtree::QuadTree::of_capacity(min_lon, min_lat,
                                max_lon-min_lon, max_lat-min_lat, 2000)
        }
    }

    pub fn squares(&self) -> impl Iterator<Item=&quadtree::QuadCell<NodePos>> + '_ {
        //XXX hash(7)
        self.by_pos.walk()
    }

    //XXX refactor? used for debugging now
    pub fn square_for<'a>(&'a self, lat: f64, lon: f64) -> &'a quadtree::QuadCell<NodePos> {
        self.by_pos.get_cell(lon, lat)
    }


    pub fn insert(&mut self, node: &Node) {
        self.by_pos.insert(NodePos {
            lat: node.lat,
            lon: node.lon,
            id: node.id
        });
        node.address().map(|addr| {
            match self.by_name.get_mut(&addr.to_lowercase()) {
                Some(vec) => vec.push(node.id),
                None => {
                    self.by_name.insert(addr.to_lowercase(), vec![node.id]);
                }
            }
        });
        if node.is_poi() {
            // Only insert nodes names with POI otherwise we insert
            // road signs and stuff
            node.name()
                .map(|name|
                     match self.by_name.get_mut(&name.to_lowercase()) {
                         Some(vec) => vec.push(node.id),
                         None => {
                             self.by_name.insert(name.to_lowercase(), vec![node.id]);
                         }
                     });
        }
        for poi in node.poi_types() {
            self.by_poi.entry(poi.to_lowercase().to_string())
                .or_insert(Vec::new())
                .push(node.id);
        }
    }

    pub fn around<'a>(&'a self, lat: f64, lon: f64) -> impl Iterator<Item=i64> + 'a {
        // Nearest with distance: using square root
        self.by_pos.get(lon, lat)
            .inspect(|n| trace!("nearest inspecting {:?}", n))
            .map(|node| node.id)
    }

    pub fn poi_types(&self) -> Vec<&String> {
        self.by_poi.keys().collect()
    }

    pub fn matching(&self, query: &String) -> Vec<i64> {
        match self.by_name.get_raw_descendant(&query.to_lowercase()) {
            Some(tree) => tree.values().flatten().cloned().collect(),
            None => vec![]
        }
    }

    pub fn of_poi<'a>(&'a self, poi: &String) -> &'a Vec<i64> {
        self.by_poi.get(&poi.to_lowercase()).unwrap()
    }

    pub fn named<'a>(&'a self) -> impl Iterator<Item=&i64> + 'a {
        self.by_name.values().flatten()
    }
}
