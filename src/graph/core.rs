use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Node {
    id: usize,
    pub neighbours: BTreeSet<usize>,
}

impl Node {
    pub fn new(id: usize, neighbours: Vec<usize>) -> Self {
        Self {
            id,
            neighbours: BTreeSet::from_iter(neighbours),
        }
    }
}

#[derive(Clone)]
pub struct Graph {
    pub nodes: BTreeMap<usize, Node>,
}

impl Graph {
    pub fn new(adjacency: Vec<Node>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(|node| (node.id, node)).collect(),
        }
    }

    pub fn induce(&self, s: usize) -> Self {
        let mut nodes = self.nodes.clone().split_off(&s);

        for node in nodes.values_mut() {
            node.neighbours = node.neighbours.split_off(&s);
        }

        Self { nodes }
    }

    pub fn subgraph(&self, subset: &[usize]) -> Self {
        let mut nodes = self.nodes.clone();

        // [PERF] Use HashSet to avoid expensive linear search per node
        nodes.retain(|id, node| {
            node.neighbours.retain(|id| subset.contains(id));
            subset.contains(id)
        });

        Self { nodes }
    }
}
