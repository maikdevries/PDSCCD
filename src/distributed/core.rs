use crate::distributed::tarjan::{Partial, Tarjan};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Protocol<'a> {
    participants: HashMap<&'static str, &'a Participant>,
}

impl<'a> Protocol<'a> {
    pub fn new(participants: Vec<&'a Participant>) -> Self {
        Self {
            participants: participants.into_iter().map(|p| (p.id, p)).collect(),
        }
    }
}

pub struct Participant {
    id: &'static str,
    graph: Graph,
}

impl Participant {
    pub fn new(id: &'static str, graph: Graph) -> Self {
        Self { id, graph }
    }

    pub fn detect(&self) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Partial>>) {
        let nodes = self
            .graph
            .nodes
            .iter()
            .filter(|(_, node)| {
                matches!(node.location, Location::External(_)) && node.neighbours.len() > 0
            })
            .map(|(id, _)| id)
            .collect();

        return Tarjan::new(&self.graph).detect(nodes);
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
