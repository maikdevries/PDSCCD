use std::collections::{BTreeMap, BTreeSet};

struct Participant {
    id: &'static str,
    graph: Graph,
}

impl Participant {
    pub fn new(id: &'static str, graph: Graph) -> Self {
        Self { id, graph }
    }

    fn compute() {
        todo!(
            "Participant should be able to compute components within their graph; the search is rooted in either some \
            specific (external) node, or any of its (external) nodes."
        )
    }

    fn receive() {
        todo!(
            "Participant should be able to receive queries from other participants; this either results in a search \
            rooted at the specific node, or the establishment of a component it previously send a query for."
        )
    }

    fn send() {
        todo!(
            "Participant should be able to send queries to other participants to have them search their graphs from \
            the sink external node."
        )
    }
}

pub struct Graph {
    pub nodes: BTreeMap<usize, Node>,
}

impl Graph {
    pub fn new(adjacency: Vec<Node>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    id: usize,
    pub location: Location,
    pub neighbours: BTreeSet<usize>,
}

impl Node {
    pub fn new(id: usize, location: Location, neighbours: Vec<usize>) -> Self {
        Self {
            id,
            location,
            neighbours: BTreeSet::from_iter(neighbours),
        }
    }
}

pub enum Location {
    External(&'static str),
    Internal,
}
