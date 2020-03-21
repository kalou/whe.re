#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;
extern crate env_logger;

mod api;
mod reduce;
mod converters;

use std::time::{Duration, Instant};
/*
use graph;
use osm;
*/

use rocket_cors::CorsOptions;

pub fn main() {
    env_logger::init();

    let mut osm_db = osm::Db::new("map.osm.pbf", vec!["gtfs/gtfs.zip"]);

    /* Alamo Square Cafe to Family
    let node0 = osm_db.node_by_id(65325380).unwrap();
    let node1 = osm_db.node_by_id(65338254).unwrap();
    // Two points on Terra Vista accross Barcelona
    let node0 = osm_db.node_by_id(65372029).unwrap();
    let node1 = osm_db.node_by_id(65372032).unwrap();

    let start = Instant::now();
    let walker = walkers::search::Search::new(node1);
    let walk_result0 = graph::walk(&osm_db, walker, node0);
    debug!("result nr inv {:?} in {:?}", walk_result0.nr_inv, start.elapsed());
    let walk_result1 = graph::walk(&osm_db, walker, node0);
    debug!("result nr inv {:?} in {:?}", walk_result1.nr_inv, start.elapsed());
    let walk_result2 = graph::walk(&osm_db, walker, node0);
    debug!("result nr inv {:?} in {:?}", walk_result2.nr_inv, start.elapsed());
    let path = walk_result2.path(node1);
    debug!("walker path {:?}->{:?} in {:?}, {} nodes",
                      node0, node1, start.elapsed(), path.steps.len());
    */

    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::ignite()
        .manage(osm_db)
        .attach(cors)
        .mount("/graph", routes![api::isochrone,
                                 api::search,
                                 api::path,
                                 api::pois,
                                 api::nodes_of_poi,
                                 api::multi_isochrone,
                                 api::square,
                                 api::score])
        .launch();
}
