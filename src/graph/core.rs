use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn empty() -> Self {
        Self {
            nodes: BTreeMap::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_initialisation() {
        let node = Node::new(2, vec![3, 5, 7]);

        assert_eq!(node.id, 2);
        assert_eq!(node.neighbours, BTreeSet::from([3, 5, 7]));
    }

    #[test]
    fn graph_initialisation() {
        let a = Node::new(2, vec![3, 5]);
        let b = Node::new(3, vec![5]);
        let c = Node::new(5, vec![2]);

        let graph = Graph::new(vec![a.clone(), b.clone(), c.clone()]);

        assert_eq!(graph.nodes.get(&a.id), Some(&a));
        assert_eq!(graph.nodes.get(&b.id), Some(&b));
        assert_eq!(graph.nodes.get(&c.id), Some(&c));
    }

    #[test]
    fn graph_empty_initialisation() {
        let graph = Graph::empty();

        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn graph_induce_nodes() {
        let a = Node::new(2, vec![]);
        let b = Node::new(3, vec![]);
        let c = Node::new(5, vec![]);

        let graph = Graph::new(vec![a.clone(), b.clone(), c.clone()]).induce(b.id);

        assert!(!graph.nodes.contains_key(&a.id));
        assert!(graph.nodes.contains_key(&b.id));
        assert!(graph.nodes.contains_key(&c.id));
    }

    #[test]
    fn graph_induce_node_neighbours() {
        let a = Node::new(5, vec![2, 7]);
        let b = Node::new(7, vec![3, 11]);
        let c = Node::new(11, vec![2, 3, 5]);

        let graph = Graph::new(vec![a.clone(), b.clone(), c.clone()]).induce(a.id);

        assert_eq!(
            graph.nodes.get(&a.id).unwrap().neighbours,
            BTreeSet::from([7]),
        );
        assert_eq!(
            graph.nodes.get(&b.id).unwrap().neighbours,
            BTreeSet::from([11]),
        );
        assert_eq!(
            graph.nodes.get(&c.id).unwrap().neighbours,
            BTreeSet::from([5]),
        );
    }

    #[test]
    fn graph_subgraph_nodes() {
        let a = Node::new(2, vec![]);
        let b = Node::new(3, vec![]);
        let c = Node::new(5, vec![]);

        let graph = Graph::new(vec![a.clone(), b.clone(), c.clone()]).subgraph(&[a.id, c.id]);

        assert!(graph.nodes.contains_key(&a.id));
        assert!(!graph.nodes.contains_key(&b.id));
        assert!(graph.nodes.contains_key(&c.id));
    }

    #[test]
    fn graph_subgraph_nodes_empty() {
        let a = Node::new(2, vec![]);
        let b = Node::new(3, vec![]);
        let c = Node::new(5, vec![]);

        let graph = Graph::new(vec![a.clone(), b.clone(), c.clone()]).subgraph(&[]);

        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn graph_subgraph_node_neighbours() {
        let a = Node::new(2, vec![3, 7, 11]);
        let b = Node::new(3, vec![5, 13]);
        let c = Node::new(5, vec![2, 3, 7]);

        let graph =
            Graph::new(vec![a.clone(), b.clone(), c.clone()]).subgraph(&[a.id, b.id, c.id, 13]);

        assert_eq!(
            graph.nodes.get(&a.id).unwrap().neighbours,
            BTreeSet::from([3]),
        );
        assert_eq!(
            graph.nodes.get(&b.id).unwrap().neighbours,
            BTreeSet::from([5, 13]),
        );
        assert_eq!(
            graph.nodes.get(&c.id).unwrap().neighbours,
            BTreeSet::from([2, 3]),
        );
    }
}
