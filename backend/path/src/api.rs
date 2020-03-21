use geojson;
use graph;
use osm;

use super::converters::ApiFrom;

use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::thread;

use super::reduce::Reduce;
use osm::walkers;
use rocket::State;

use rocket_contrib::json::Json;

use serde::Deserialize;
use serde::Serialize;

const INITIAL_DIST:u64=150; // XXX this is a temp non optimal way to find
                          // nodes with a connection to the graph from a POI

// Api types
//
// Inputs:

#[derive(Debug, Deserialize)]
// A single point - by Lat/Lon or node_id
// that can be used as a starting point.
// If we allowed POIs this would mean adding lon/lat or search boundaries
// and working with arrays of potential nodes - instead keep the logic sane
// with a single point of entry
pub enum Point {
    LonLat {
        lon: f64,
        lat: f64,
    },

    Node {
        node_id: i64,
    },
}

impl Point {
    fn get_node<'a>(&self, db: &'a osm::Db) -> Option<&'a osm::Node> {
         match &self {
            Point::Node { node_id } => db.node_by_id(*node_id),
            Point::LonLat { lon, lat } =>
                db.initial_node(*lat, *lon, INITIAL_DIST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum PoiConstraint {
    OnTheWay(Point, Point),
    Near(Point, u64),
    DefinedByPoi(String, u64), // Slow, find poi then discover point
    NearPoi(String, u64), // "Faster" check node is within range of POI
}

#[derive(Debug, Deserialize)]
pub struct MultiIc {
    poi: String,
    // This is almost optional since we'd start from the constraints
    // for most cases. Exception being: gimme a bar near a market.
    constraints: Vec<PoiConstraint>
}

#[derive(Debug, Deserialize)]
pub struct Score {
    constraints: Vec<PoiConstraint>
}

#[derive(Debug, Serialize)]
pub struct NodeDescription {
    node_id: i64,
    name: Option<String>,
    address: Option<String>,
    lon: f64,
    lat: f64,
    kind: Vec<String>,
    wheelchair: Option<bool>
}

impl ApiFrom<&osm::Node> for NodeDescription {
    fn api_from(node: &osm::Node) -> Self {
        NodeDescription {
            node_id: node.id,
            name: node.name(),
            address: node.address(),
            lon: node.lon,
            lat: node.lat,
            kind: node.poi_types().into_iter().map(|s| s.to_string()).collect(),
            wheelchair: node.wheelchair()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MultiIcResult {
    // The points matching the query
    points: Vec<NodeDescription>, // the matching pois
    nodes: Vec<NodeDescription>, // the specified nodes
    paths: geojson::FeatureCollection<geojson::MultiLine>
    // TODO Add shit here
}

#[derive(Debug, Serialize)]
// This is a square on a map, with a score
pub struct SquareScore {
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
    score: u64,
    features: Vec<String>,
    costs: HashMap<String, u64>
}

#[derive(Debug, Serialize)]
pub struct ScoreResult {
    squares: Vec<SquareScore>
}

#[get("/path?<from>&<to>")]
pub fn path(state: State<osm::Db>, from: i64, to: i64) -> Json<geojson::FeatureCollection<geojson::MultiLine>>
{
    let osm = state.inner();

    let from = osm.node_by_id(from).unwrap();
    let to = osm.node_by_id(to).unwrap();

    let walker = walkers::explore::Explore {
        max_cost: None,
        predicate: |x| x == to,
        target: Some(to)
    };

    let features = graph::walk(osm, walker, from).steps.iter().map(|step| 
        geojson::Feature{
            properties: geojson::Properties {
                name: Some(format!("cost {}", step.total)),
                node_id: Some(step.to.id)
            },
            data: geojson::MultiLine::api_from(step)
        }
    ).collect();
    Json(geojson::FeatureCollection{ features })
}

#[get("/isochrone?<node>&<dist>")]
pub fn isochrone(state: State<osm::Db>, node: i64, dist: u64)
    -> Json<geojson::MultiLine> {
    let osm = state.inner();

    let walker = walkers::explore::Explore {
        max_cost: Some(dist),
        predicate: |x| true,
        target: None
    };

    let node = osm.closest_initial(node, INITIAL_DIST).unwrap();
    Json(geojson::MultiLine::api_from(graph::walk(osm, walker, node)))
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    nodes: Vec<NodeDescription>
}

#[get("/search?<q>&<lat>&<lon>")]
pub fn search(state: State<osm::Db>, q: String, lat: f64, lon: f64) -> Json<SearchResult> {
    let osm = state.inner();

    let mut nodes = osm.node_index.matching(&q)
            .into_iter()
            .filter_map(|id| osm.node_by_id(id))
            .collect::<Vec<&osm::Node>>();
    nodes.sort_by(|a, b| a.distance_from(lat, lon).cmp(
                                &b.distance_from(lat, lon)));

    // XXX Inject latency for test
    thread::sleep(Duration::from_millis(300));

    Json(SearchResult {
        nodes: nodes.iter()
            .take(10)
            .map(|node| NodeDescription::api_from(node))
            .collect()
    })
}

#[derive(Serialize)]
pub struct PoiList {
    items: Vec<String>,
}

#[get("/pois")]
pub fn pois(state: State<osm::Db>) -> Json<PoiList> {
    let osm = state.inner();

    // XXX Inject latency for test
    thread::sleep(Duration::from_millis(300));

    Json(PoiList {items: osm.node_index.poi_types().iter()
        .map(|s| s.to_string())
        .collect()})
}

#[get("/pois/<kind>?<lat>&<lon>&<dist>")]
pub fn nodes_of_poi(state: State<osm::Db>, kind: String,
                    lat: f64, lon: f64, dist: u64) -> Json<SearchResult> {
    let osm = state.inner();
    //XXX We should use quadmap for spatial index queries instead
    Json(SearchResult {
        nodes: osm.node_index.of_poi(&kind)
            .into_iter()
            .filter_map(|id| osm.node_by_id(*id))
            .filter(|node| node.distance_from(lat, lon) < dist)
            .map(|node| NodeDescription::api_from(node))
            .collect()
    })
}

#[post("/isochrone", data = "<params>")]
pub fn multi_isochrone(state: State<osm::Db>, params: Json<MultiIc>)
    -> Json<MultiIcResult> {
    let osm = state.inner();

    debug!("Multi Ic looking for {}", params.poi);

    let mut res_vec = Vec::new();
    let mut poi_vec = Vec::new(); // The "nearPOI" is a big OR of all
                                // matching POIs that we add to the other
                                // constraints

    // First iterate through Near types
    let mut nodes = HashSet::new();
    let mut pois = HashSet::new();
    let predicate = |a: &osm::Node| a.is_poi_type(&params.poi);

    for c in &params.constraints {
        match c {
            PoiConstraint::Near(point, max_cost) => {
                let point = point.get_node(&osm).unwrap();
                let walker = walkers::explore::Explore {
                    max_cost: Some(*max_cost),
                    predicate: predicate,
                    target: None
                };
                let res = graph::walk(osm, walker, &point);
                debug!("walking from {:?} cost {}", point, res.nr_inv);
                res_vec.push(res);
                nodes.insert(point);
            },
            /* This scans too many POIs */
            PoiConstraint::DefinedByPoi(kind, max_cost) => {
                for poi in osm.node_index.of_poi(&kind).iter()
                    .filter_map(|id| osm.node_by_id(*id)) {
                    let walker = walkers::explore::Explore {
                        max_cost: Some(*max_cost),
                        predicate: predicate, /* Needed for type -
                                                 wait for a refactor
                                                 */
                        target: None
                    };
                    let res = graph::walk(osm, walker, &poi);
                    debug!("walking from POI {:?} cost {}", poi, res.nr_inv);
                    if (res.steps.iter().any(|x| predicate(x.to))) {
                        debug!("POI {:?} matches NearPoi", poi);
                        pois.insert(poi);
                    }
                    poi_vec.push(res);
                }
            },
            PoiConstraint::OnTheWay(a, b) => {
                let a = a.get_node(&osm).unwrap();
                let b = b.get_node(&osm).unwrap();
                let walker = walkers::explore::Explore {
                    max_cost: None,
                    predicate: predicate,
                    target: Some(&b)
                };
                let res = graph::walk(osm, walker, &a);
                debug!("tracing from {:?} to {:?} cost {}", a, b, res.nr_inv);
                res_vec.push(res);
                nodes.insert(a);
                nodes.insert(b);
            },
            _ => {}
        }
    }

    if let Some(defining_pois) = poi_vec.into_iter().reduce(|a, b| (a | b)) {
        res_vec.push(defining_pois);
    }

    let explored = res_vec.into_iter().reduce(|a, b| a & b);


    // Here we have points satisfying OnTheWay & Near
    let steps = match explored {
        Some(res) => res.steps,
        None => vec![]
    };

    let mut features = Vec::new();

    // And also: verify for each point that it has a wanted POI nearby,
    // if that is on our list
    steps.iter().filter(|node| {
        // Filter out NearPoi first
        params.constraints.iter()
        .map(|c| {
            match c {
                PoiConstraint::NearPoi(kind, cost) => {
                    let walker = walkers::explore::Explore {
                                max_cost: Some(*cost),
                                predicate: |x| x.is_poi_type(&kind),
                                target: None,
                    };
                    let res = graph::walk(osm, walker, node.to);
                    res.steps.iter().map(|step| {
                        debug!("NearPoi {:?} OK for {:?} => {:?}", c,
                               &node.to, &step.to);
                        nodes.insert(step.to);
                        true
                    }).any(|x| x == true)
                }
                _ => true
            }
        }).all(|x| x == true)
    }).filter(|a| a.to.is_poi_type(&params.poi))
        .inspect(|a| debug!("Adding Exp from step to {:?}", a.to))
        .for_each(|a| {
            pois.insert(a.to);
            features.push(geojson::Feature{
                properties: geojson::Properties {
                    name: Some(format!("{:?} (cost {})", a.to.name(),
                    a.total)),
                    node_id: Some(a.to.id)
                },
                data: geojson::MultiLine::api_from(a)
            });
        });

    Json(MultiIcResult {
        points: pois.iter().map(|a| NodeDescription::api_from(a)).collect(),
        nodes: nodes.iter().map(|a| NodeDescription::api_from(a)).collect(),
        paths: geojson::FeatureCollection {
            features
        }
    })
}

#[post("/square?<lat>&<lon>")]
pub fn square(state: State<osm::Db>, lat: f64, lon: f64) -> Option<Json<SquareScore>>
{
    let osm = state.inner();

    let n = osm.scores.get(&osm.node_index.square_for(lat, lon).id);
    n.map(|score| Json(SquareScore {
        left: score.left,
        right: score.right,
        bottom: score.bottom,
        top: score.top,
        score: 0,
        features: score.scores.keys().cloned().collect(),
        costs: score.scores.clone()
    }))
}

#[post("/score", data = "<params>")]
pub fn score(state: State<osm::Db>, params: Json<Score>) -> Json<ScoreResult>
{
    let osm = state.inner();

    let mut squares = HashSet::new();
    let mut wanted_pois = HashMap::new();
    let mut node_cons = Vec::new();

    params.constraints.iter().for_each(|cons| match cons {
        PoiConstraint::NearPoi(kind, cost) => {
            wanted_pois.insert(kind, cost);
        },
        PoiConstraint::Near(node, cost) => {
            node_cons.push((node, cost));
        },
        cons => {
            warn!("ignoring constraint {:?}", cons);
        }
    });

    // TODO Poi costs, scores, square_for, etc. has no interface in OSM,
    // create one instead of raw accessing everything
    node_cons.iter()
        .map(|(point, dist)| {
            let node = point.get_node(&osm).unwrap();
            let walker = walkers::explore::Explore {
                max_cost: Some(**dist),
                predicate: |_| true,
                target: None
            };
            graph::walk(osm, walker, node)
        }).reduce(|a, b| a&b)
        .map(|res| res.steps.iter().for_each(|step| {
            squares.insert(
                osm.node_index.square_for(step.to.lat, step.to.lon).id
            );
        }));

    // TODO Again here - hardcoded 1500,...
    let score_fn = |square: &osm::SquareScore| {
        wanted_pois.iter()
            .map(|(poi, max_dist)| *max_dist - square.scores.get(*poi).unwrap_or(&max_dist))
            // Each poi is worth % of dist - ie 25% if 4 pois
            .map(|dist| (100.0 * dist as f64) / (1500.0
                    * wanted_pois.len() as f64))
            .map(|float_pct| float_pct as u64)
            .sum()
    };

    Json(ScoreResult {
        squares: squares.iter().filter_map(|n| osm.scores.get(&n))
            .map(|n| SquareScore {
                left: n.left,
                right: n.right,
                bottom: n.bottom,
                top: n.top,
                score: score_fn(n),
                features: n.scores.keys().cloned().filter(|key|
                              wanted_pois.contains_key(&key)
                ).collect(),
                costs: n.scores.clone()
            }).collect()
    })
}
