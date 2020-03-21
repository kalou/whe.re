use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::{Duration, Instant};
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::vec::Vec;
use std::rc::Rc;
use std::fmt;

#[macro_use]
extern crate log;

pub trait Graph: Sized {
    type Node: Hash + Eq + Debug;
    type Edge: Debug;
}

// A walker state needs clone/copy - try to keep it small?
pub trait GraphWalker<'a, G: Graph> : Sized {
    type State: Debug + Sized + Clone;

    // Get neighbors and cost to join them for state
    fn neighbors(&self, graph: &'a G, prev: &Step<'a, G, Self>) ->
        // Return type being Way - cost, heuristic, node, state
        Vec<(&'a G::Edge, u64, u64, &'a G::Node, Self::State)>;

    fn initial_state(&self, node: &G::Node) -> Self::State;

    // The same graph can be replicated on an extra dimension, so we can
    // have state transitions that reuse some already visited nodes with
    // the new state. The state is the key to switch from one graph to
    // the other.
    fn graph_id(&self, state: &Self::State) -> u64;

    // Stop condition handled by walker
    fn stop(&self, res: &WalkResult<'a, G, Self>) -> bool;

    // Should we include this as a step in the result set ?
    fn include(&self, step: &Step<'a, G, Self>) -> bool;
}

#[derive(Debug)]
pub struct Step<'a, G: Graph, W: GraphWalker<'a, G>> {
    pub from: Option<Rc<Step<'a, G, W>>>, // Node_id
    pub edge: Option<&'a G::Edge>, // Edge used
    pub cost: u64,                 // cost for this step
    pub total: u64,                // total cost so far
    pub total_f: u64,              // cost + heuristic for A*
    pub to: &'a G::Node,           // target
    pub state: W::State            // cloned
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> fmt::Display for Step<'a, G, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.from {
            Some(prev) => write!(f, "Step from {:?}", prev.to)?,
            None => write!(f, "?")?
        }
        write!(f, "->{:?}", self.to)?;
        write!(f, " (total {}+{}) ({:?})", self.total, self.cost, self.state)
    }
}

// PartialOrd for the priority queue
impl<'a, G: Graph, W: GraphWalker<'a, G>> PartialOrd for Step<'a, G, W> {
    fn partial_cmp(&self, other: &Step<'a, G, W>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> Ord for Step<'a, G, W> {
    fn cmp(&self, other: &Step<'a, G, W>) -> Ordering {
        // Reversed for a minqueue
        other.total_f.cmp(&self.total_f)
    }
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> PartialEq for Step<'a, G, W> {
    fn eq(&self, other: &Step<'a, G, W>) -> bool {
        //XXX
        self.total == other.total && self.to == self.to
    }
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> Eq for Step<'a, G, W> {}

#[derive(Debug)]
pub struct WalkResult<'a, G: Graph, W: GraphWalker<'a, G>> {
    pub steps: Vec<Rc<Step<'a, G, W>>>,
    pub nr_inv: u64 // Number of steps to build the graph (debug)
}

// neighbors() has the previous step with state
// giving us a set of potential next steps with cost estimated.
// this way we can:
//  - wait for a bus (ie: 15mn + 1mn to reach next node of bus line)
//  - be on a bus and simply run at bus speed (making it interesting to
//          wait for one in the previous step)
//  - find a car and start driving
//  - leave the car because there's a parking
//  - not turn right cause constraint
//  - etc.

struct DistanceWithGraph<'a, G: Graph> {
    node: &'a G::Node,
    graph_id: u64
}

impl<'a, G: Graph> Eq for DistanceWithGraph<'a, G> {}
impl<'a, G: Graph> PartialEq for DistanceWithGraph<'a, G> {
    fn eq(&self, other: &DistanceWithGraph<'a, G>) -> bool {
        //XXX
        self.node == other.node &&
            self.graph_id == other.graph_id
    }
}
impl<'a, G: Graph> Hash for DistanceWithGraph<'a, G> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.graph_id.hash(state);
    }
}

// Visit nodes, relaxing distances
pub fn walk<'a, G: Graph, W: GraphWalker<'a, G>>(graph: &'a G, walker: W, start: &'a G::Node) -> WalkResult<'a, G, W> {
    let mut queue = BinaryHeap::new();
    let mut distances = HashMap::<DistanceWithGraph<'a, G>, u64>::new();

    let mut res = WalkResult{ steps: Vec::new(), nr_inv: 0 };
    let start_time = Instant::now();

    queue.push(Rc::new(Step{
        from: None,
        edge: None,
        cost: 0,
        total: 0,
        total_f: 0, // As in f = g+h
        to: start,
        state: walker.initial_state(start)}));

    while let Some(step) = queue.pop() {
        // XXX Is this still needed?
        let known = distances.get(&DistanceWithGraph{
            graph_id: walker.graph_id(&step.state),
            node: &step.to
        });

        if walker.stop(&res) {
            break
        }

        match known {
            Some(&distance) if step.total > distance => {
                continue;
            }
            _ => {}
        }
        // EOXXX -- walker.stop() is needed.

        res.nr_inv += 1;

        for (edge, w, h, node, new_state) in walker.neighbors(graph, &step) {
            let total = step.total + w;
            let known = distances.entry(DistanceWithGraph{
                graph_id: walker.graph_id(&new_state),
                node: node
            }).or_insert(total + 1);
            trace!("from {:?} potential {:?} {}+{} {:?}", step.to,
                   node, w, h, new_state);

            // Relax if below
            // The closest will pop out first
            if total < *known {
                *known = total;

                let new_step = Rc::new(Step {
                    from: Some(Rc::clone(&step)),
                    edge: Some(edge),
                    cost: w,
                    total: total,
                    total_f: total + h,
                    to: node,
                    state: new_state,
                });

                if walker.include(&new_step) {
                    res.steps.push(Rc::clone(&new_step));
                }

                queue.push(new_step);
            }

       }
    }

    debug!("Walker from {:?} in {:?}, cost {}", start, start_time.elapsed(),
        res.nr_inv);

    res
}

use std::ops::{BitAnd, BitOr};

// WalkResult a & b => steps that have b in common
impl<'a, G: Graph, W: GraphWalker<'a, G>> BitAnd for WalkResult<'a, G, W> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let nodes = self
            .steps
            .iter()
            .map(|step| step.to)
            .collect::<HashSet<&G::Node>>();
        let other = rhs
            .steps
            .iter()
            .map(|step| step.to)
            .collect::<HashSet<&G::Node>>();
        let common = nodes
            .intersection(&other)
            .cloned()
            .collect::<HashSet<&G::Node>>();

        Self {
            steps: self
                .steps
                .into_iter()
                .filter(|step| common.contains(step.to))
                .collect(),
            nr_inv: self.nr_inv
        }
    }
}

// WalkResult a | b => merge both
impl<'a, G: Graph, W: GraphWalker<'a, G>> BitOr for WalkResult<'a, G, W> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            steps: self.steps.into_iter()
                    .chain(rhs.steps.into_iter())
                    .collect(),
            nr_inv: self.nr_inv
        }
    }
}

pub struct ReverseStepIterator<'a, G: Graph, W: GraphWalker<'a, G>> {
    current: Option<&'a Step<'a, G, W>>
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> Iterator
	for ReverseStepIterator<'a, G, W> {
    type Item = &'a Step<'a, G, W>;

    fn next(&mut self) -> Option<&'a Step<'a, G, W>> {
        self.current.map(|cur| {
            self.current = cur.from.as_ref().map(|x| x.as_ref());
            cur
        })
    }
}

impl<'a, G: Graph, W: GraphWalker<'a, G>> IntoIterator
    for &'a Step<'a, G, W> {
    type Item = &'a Step<'a, G, W>;
    type IntoIter = ReverseStepIterator<'a, G, W>;

    fn into_iter(self) -> ReverseStepIterator<'a, G, W> {
        ReverseStepIterator{ current: Some(self) }
    }
}
