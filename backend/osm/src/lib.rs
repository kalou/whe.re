#[macro_use]
extern crate log;
extern crate env_logger;
extern crate pbf_reader;

use pbf_reader::PBFData;

use std::sync::mpsc;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Instant;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use gtfs::{TransitMap, Stop};

use serde::Deserialize;

pub mod walkers;
mod node_index;
mod poi;

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    k: String,
    v: String,
}

trait OsmTags {
    fn has_key(&self, k: &str) -> bool;
    fn get_key<'a>(&'a self, k: &str) -> Option<&'a String>;
}

impl OsmTags for Vec<Tag> {
    fn has_key(&self, k: &str) -> bool {
        for tag in self {
            if tag.k == k {
                return true;
            }
        }
        return false;
	}

    fn get_key<'a>(&'a self, k: &str) -> Option<&'a String> {
        for tag in self {
            if tag.k == k {
                return Some(&tag.v);
            }
        }
        return None;
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
    tags: Vec<Tag>
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Node {
    // Returns the distance in meters
    pub fn distance_from(&self, lat: f64, lon: f64) -> u64 {
        let phi1 = self.lat.to_radians();
        let phi2 = lat.to_radians();
        let dphi = (lat - self.lat).to_radians();
        let dlam = (lon - self.lon).to_radians();
        let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlam / 2.0).sin().powi(2);
        let c = 2.0 * (a.sqrt().atan2((1.0 - a).sqrt()));
        let distance = 6371000.0 * c; //  mean radius of earth
        return distance as u64;
    }

    pub fn distance(&self, other: &Node) -> u64 {
        self.distance_from(other.lat, other.lon)
    }

    pub fn poi_types(&self) -> Vec<String> {
        self.tags.iter().filter(|tag| vec!["amenity",
                    "shop", "leisure", "sport", "tourism",
                    "information", "natural"].contains(&tag.k.as_str()))
            .map(|tag| tag.v.to_string())
            .collect()
    }

    pub fn is_poi(&self) -> bool {
        self.poi_types().len() > 0
    }

    pub fn is_poi_type(&self, s: &String) -> bool {
        self.poi_types().contains(&s)
    }

    pub fn wheelchair(&self) -> Option<bool> {
        self.tags.get_key("wheelchair").map(|x| x == "yes")
    }

    pub fn name(&self) -> Option<String> {
        self.tags.get_key("name").map(|s| s.to_string())
    }

    // TODO: create type address
    pub fn address(&self) -> Option<String> {
        vec![self.tags.get_key("addr:housenumber"),
            self.tags.get_key("addr:street"),
            self.tags.get_key("addr:city")]
            .into_iter()
            .map(|s| match s {
                Some(s) => Some(s.to_string()),
                None => None
            })
            .collect::<Option<Vec<String>>>()
            .map(|str_list| str_list.join(" "))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Nd {
    // Just a node id
    #[serde(rename = "ref", default)]
    pub id: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Way {
    pub id: i64,
    #[serde(rename = "nd", default)]
    pub nodes: Vec<i64>,
    #[serde(rename = "tag", default)]
    tags: Vec<Tag>,
}

impl Way {
    pub fn name(&self) -> Option<&String> {
        self.tags.get_key("name")
    }

    //XXX We artificially connect nodes for graph with 1
    pub fn is_highway(&self) -> bool {
        self.tags.has_key("highway") || self.id == 1
    }

    pub fn is_cycleway(&self) -> bool {
        self.tags.has_key("cycleway")
            || self.tags.has_key("cycleway:left")
            || self.tags.has_key("cycleway:right")
    }

    // TODO: extend "way_ok(nodeA, nodeB)?
    pub fn is_oneway(&self) -> bool {
        match self.tags.get_key("oneway") {
            Some(s) => match s.as_str() {
                "yes" => true,
                "no" => false,
                v => {
                    warn!("unknown oneway type {}", v);
                    true
                }
            },
            None => false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Member {
    // For a relation
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(rename = "ref", default)]
    ref_id: i64,
    role: String,
}

#[derive(Debug, Deserialize)]
pub struct Relation {
    id: i64,
    #[serde(rename = "member", default)]
    members: Vec<Member>,
    #[serde(rename = "tag", default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct BoundingBox {
    minlat: f64,
    minlon: f64,
    maxlat: f64,
    maxlon: f64
}

type Adjacency = (i64, u64, i64);
#[derive(Debug)]
struct AdjacencyMap(HashMap<i64, Vec<Adjacency>>);

pub struct SquareScore {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub scores: HashMap<String, u64>
}

pub struct Db {
    nodes: HashMap<i64, Node>, // node_id to <nodes> index
    ways: HashMap<i64, Way>,  // way_id to <ways> index
    adjacencies: AdjacencyMap,
    transit: TransitMap,
    pub node_index: node_index::NodeIndex,
    pub scores: HashMap<u64, SquareScore>,
}

impl AdjacencyMap {
    fn connect(&mut self, a: i64, b: i64, way: i64, dist: u64) {
        self.0.entry(a)
            .or_insert(Vec::new())
            .push((way, dist, b));
        self.0.entry(b)
            .or_insert(Vec::new())
            .push((way, dist, a));
    }
    fn get(&self, key: &i64) -> Option<&Vec<Adjacency>> {
        self.0.get(key)
    }
}

impl Db {
    pub fn new(filename: &'static str, gtfs: Vec<&str>) -> Self {
        info!("Loading {}", filename);

        let (mut node_tx, node_rx) = mpsc::channel::<PBFData>();
        let h = thread::spawn(move || {
            return pbf_reader::read_pbf(&filename.to_string(), 9, &mut node_tx);
        });

        let mut index : Option<node_index::NodeIndex> = None;

        // Find a better use of this later as this is a good memory saver
        let mut pbf_ways = HashMap::new();
        let mut pbf_nodes = HashMap::new();
        let mut pbf_strings = HashMap::new();

        loop {
            match node_rx.recv().unwrap() {
                PBFData::NodesSet(set) => {
                    for (id, node) in set {
                        pbf_nodes.insert(id, node);
                    }
                }
                PBFData::WaysSet(set) => {
                    for (id, way) in set {
                        pbf_ways.insert(id, way);
                    }
                }
                PBFData::RelationsSet(_) => { }
                PBFData::Strings(id, string) => {
                    pbf_strings.insert(id, string);
                }
                PBFData::PbfInfo(inf) => {
                    if !index.is_some() {
                        debug!("Setting BBOX BL to TR x,y = {},{} {},{}",
                               inf.bbox.top_left.lon, inf.bbox.bottom_right.lat,
                               inf.bbox.bottom_right.lon, inf.bbox.top_left.lat);
                        index = Some(node_index::NodeIndex::new(
                                inf.bbox.top_left.lon, inf.bbox.bottom_right.lat,
                               inf.bbox.bottom_right.lon, inf.bbox.top_left.lat));
                    }
                }
                PBFData::ParseEnd => {
                                break;
                }
            }
        }

        let r = h.join().unwrap();

        let mut db = Db {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            adjacencies: AdjacencyMap(HashMap::new()),
            node_index: index.unwrap(),
            scores: HashMap::new(),
            transit: TransitMap::new()
        };

        // Load GTFS before we index/prewalk anything
        for file in gtfs {
            info!("Loading GTFS {}", &file);
            db.transit.load(&file);
        }

        // Artifical way for poi connections
        db.ways.insert(1, Way{
            id: 1,
            nodes: vec![],
            tags: vec![]
        });

        pbf_ways.iter().for_each(|(id, way)| {
            let strings = pbf_strings.get(&way.tags.string_table_id).unwrap();
            db.ways.insert(*id, Way {
                id: *id,
                nodes: way.nodes.iter().map(|a| *a).collect(),
                tags: way.tags.get_keys_vals(strings).iter()
                    .map(|(k, v)| Tag{
                        k: k.to_string(),
                        v: v.to_string()
                    })
                    .collect()
            });
        });

        pbf_nodes.iter().for_each(|(id, node)| {
            let strings = pbf_strings.get(&node.tags.string_table_id).unwrap();
            db.nodes.insert(*id, Node {
                id: *id,
                lat: node.coord.lat,
                lon: node.coord.lon,
                //XXX factor
                tags: node.tags.get_keys_vals(strings).iter()
                    .map(|(k, v)| Tag{
                        k: k.to_string(),
                        v: v.to_string()
                    })
                .collect()

            });
        });


        info!("Building graph");
        db.make_graph();
        info!("Indexing {} nodes", db.nodes.len());
        db.index_nodes();
        info!("Connecting nodes");
        db.connect_nodes();
        info!("Preparing POI score map");

        let mut scores = HashMap::new();
        // Now for each square evaluate ~center dist from POIs
        // by walking from the center and logging POIs as we see them
        for cell in db.node_index.squares() {
            if let Some(node) = db.initial_node(cell.y+cell.ysize/2.0,
                                                cell.x+cell.xsize/2.0, 500) {
                let walker = walkers::explore::Explore {
                    max_cost: Some(1500),
                    predicate: |x| x.is_poi(),
                    target: None
                };
                let res = graph::walk(&db, walker, &node);
                for step in &res.steps {
                    step.to.poi_types().iter().for_each(|pt| {
                        let cellmap = scores.entry(cell.id).or_insert(
                            SquareScore {
                                top: cell.y + cell.ysize,
                                bottom: cell.y,
                                left: cell.x,
                                right: cell.x + cell.xsize,
                                scores: HashMap::new()
                            }
                        );
                        let dist = cellmap.scores.entry(pt.to_string())
                            .or_insert(step.total);
                        *dist = step.total.min(*dist);
                    });
                }
            }
        }

        db.scores = scores;
        db
    }

    pub fn connect_nodes(&mut self) {
        let mut connected = 0;
        for node in self.nodes.values() {
            if node.is_poi() || node.name().is_some() {
                connected += 1;
                let gnode = self.node_index.around(node.lat, node.lon)
                    .filter(|id| self.adjacencies.get(&id).is_some())
                    .filter_map(|id| self.node_by_id(id))
                    .find(|n| n.distance_from(node.lat, node.lon) < 35)
                    .map(|n| n.id);

                match gnode {
                    Some(b) => {
                        self.adjacencies.connect(node.id, b, 1, 1)
                    },
                    None => {}
                }
            }
        }
        info!("Done connected {} nodes", connected);
    }

    pub fn make_graph(&mut self) {
        for way in self.ways.values() {
            // Only connect walkable nodes
            if !way.is_highway() {
                continue;
            }

            if way.nodes.len() < 2 {
                warn!("One-node way {:?}", way);
                continue;
            }

            for (a, b) in way.nodes.iter().zip(way.nodes[1..].iter()) {
                if let Some(a) = self.node_by_id(*a) {
                    if let Some(b) = self.node_by_id(*b) {
                        let distance = a.distance(b);
                        self.adjacencies.connect(a.id, b.id, way.id, distance)
                    }
                }
            }
        }
    }

    pub fn index_nodes(&mut self) {
        for node in self.nodes.values() {
            self.node_index.insert(&node);
        }
    }

    pub fn distance(&self, a: i64, b: i64) -> u64 {
        let a = self.node_by_id(a).unwrap();
        let b = self.node_by_id(b).unwrap();
        a.distance(b)
    }

    pub fn node_by_id(&self, id: i64) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn way_by_id(&self, id: i64) -> Option<&Way> {
        self.ways.get(&id)
    }

    //XXX just testing
    pub fn transit_stop(&self, node: &Node) -> Option<&Stop> {
        // Get all <5meters stops?
        self.transit.get_stops(node.lat, node.lon).find(|stop|
                node.distance_from(stop.get_lat(), stop.get_lon()) < 1)
    }

    //XXX todo faster/lighter would help
    pub fn neighbors(&self, node: &Node) -> Vec<(&Way, u64, &Node)> {
        match self.adjacencies.get(&node.id) {
            Some(vec) => vec
                .iter()
                .map(|(way, dist, node)| {
                    (
                        self.way_by_id(*way).unwrap(),
                        *dist,
                        self.node_by_id(*node).unwrap(),
                    )
                })
                .collect(),
            _ => vec![],
        }
    }

    pub fn initial_node(&self, lat: f64, lon: f64, dist: u64) -> Option<&Node> {
        // Return the first close enough node suitable for a walk
        // XXX: the quadmap is unsorted, anything in the square comes out
        // dist is unused

        let start = Instant::now();
        let ret = self.node_index.around(lat, lon)
            .filter(|id| self.adjacencies.get(&id).is_some())
            .filter_map(|id| self.node_by_id(id))
            .find(|n| n.distance_from(lat, lon) < dist);

        trace!("initial_node for {}/{} took {:?}", lat, lon, start.elapsed());
        ret
    }

    pub fn closest_initial(&self, node: i64, dist: u64) -> Option<&Node> {
        // Same as above from a starting node
        self.node_by_id(node).and_then(|node|
             self.initial_node(node.lat, node.lon, dist)
        )
    }
}

//
// Adapt osm to graph
// XXX impl. rush check why this is required for both
impl graph::Graph for Db {
    type Node = Node;
    type Edge = Way;
}
