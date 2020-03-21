use graph;
use gtfs::TripCursor;
use super::super::{Node, Way, Db};

/* Walkers: strategy + how to walk and report data from the walked graphs */

#[derive(Debug)]
pub struct Explore<'a, P: Fn(&Node) -> bool> {
    pub max_cost: Option<u64>,
    pub predicate: P,
    pub target: Option<&'a Node>
}

#[derive(Debug, Clone, Copy)]
pub struct State<'a> {
    speed: u64,
    on_trip: Option<TripCursor<'a>>,
    found: u64
}

impl<'a, P: Fn(&Node) -> bool> graph::GraphWalker<'a, Db>
for Explore<'a, P> {
    type State = State<'a>;

    fn graph_id(&self, state: &State) -> u64 {
        0
    }

    fn initial_state(&self, n: &Node) -> State<'a> { 
        State {speed: 1, found: 0, on_trip: None}
    }

    fn stop(&self, res: &graph::WalkResult<'a, Db, Self>) -> bool {
        match self.target {
            Some(t) => res.steps.iter().any(|x| x.to == t),
            None => false
        }
    }

    fn include(&self, prev: &graph::Step<'a, Db, Self>) -> bool {
        (self.predicate)(&prev.to) || match self.target {
            Some(t) => prev.to == t,
            None => false
        }
    }

    fn neighbors(&self, osm: &'a Db,
                     prev: &graph::Step<'a, Db, Self>)
        -> Vec<(&'a Way, u64, u64, &'a Node, State<'a>)> {

            let mut neighbors = osm.neighbors(prev.to)
            .iter()
            // filter appropriate lanes
            .filter(|(way, _, _)| way.is_highway())
            // remove edges if past max distance
            .filter(|(_, b, _)| match self.max_cost {
                Some(cost) => prev.total + b <  cost,
                None => true
            }).map(|(a, b, c)| {
                let mut state = prev.state.clone();
                let h = match self.target {
                    Some(target) => c.distance(target),
                    None => 0
                };
                if (self.predicate)(c) {
                    state.found += 1;
                }

                (*a, *b/state.speed, h, *c, state)
            })
            .collect();

            // If we're riding a bus now, simply add the
            // next stop
            // TODO

            // If we're at a bus stop, maybe find the next
            // departure
            if let Some(stop) = osm.transit_stop(prev.to) {
                //TODO - wait 5mn, then push cursor into state
                for trip in stop.next_trips() {
                    // way,cost,heuristic,node,state
                    // push next stop with cost estimate + state
                    // https://lichess.org/yiZxhfusDIkR
                }
            }

 
            neighbors
    }
}
