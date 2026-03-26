use std::collections::{BTreeMap, BTreeSet};

use crate::centralised::{core::Graph, tarjan::Tarjan};

pub struct Johnson {
    B: BTreeMap<usize, BTreeSet<usize>>,
    blocked: BTreeSet<usize>,
    circuits: Vec<Vec<usize>>,
    graph: Graph,
    n: usize,
    s: usize,
    stack: Vec<usize>,
    subgraph: Graph,
}

impl Johnson {
    pub fn new(graph: Graph) -> Self {
        Self {
            B: BTreeMap::new(),
            blocked: BTreeSet::new(),
            circuits: Vec::new(),
            n: graph.nodes.last_key_value().map(|(&k, _)| k).unwrap_or(0),
            s: graph.nodes.first_key_value().map(|(&k, _)| k).unwrap_or(0),
            stack: Vec::new(),
            graph,
            subgraph: Graph::empty(),
        }
    }

    pub fn detect(mut self) -> Vec<Vec<usize>> {
        // [BUG] Does not consider components rooted in last graph node
        while self.s < self.n {
            // [NOTE] Compute strongest connected component of subgraph G induced by { s, s + 1, ..., n }
            self.graph = self.graph.induce(self.s);
            self.subgraph = {
                // [PERF] Exhaust all detected components before performing another search
                let components = Tarjan::new(&self.graph).detect();
                let component = components
                    .iter()
                    .filter(|c| c.len() > 1)
                    .min_by_key(|c| c.iter().min());

                println!("{components:?}");

                if let Some(scc) = component {
                    self.graph.subgraph(scc)
                } else {
                    Graph::new(Vec::new())
                }
            };

            if let Some((&id, _)) = self.subgraph.nodes.first_key_value() {
                self.s = id;

                for i in self.subgraph.nodes.keys() {
                    self.blocked.remove(i);
                    self.B.remove(i);
                }

                self.circuit(self.s);

                // [PERF] Set to least node not part of any previous components
                self.s += 1;
            } else {
                self.s = self.n;
            }
        }

        return self.circuits;
    }

    fn circuit(&mut self, v: usize) -> bool {
        let mut f = false;

        self.stack.push(v);
        self.blocked.insert(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        let neighbours = self.subgraph.nodes[&v].neighbours.clone();

        for w in &neighbours {
            if *w == self.s {
                let mut stack = self.stack.clone();
                stack.push(self.s);

                self.circuits.push(stack);
                f = true;
            } else if !self.blocked.contains(w) && self.circuit(*w) {
                f = true;
            }
        }

        if f {
            self.unblock(v);
        } else {
            for w in neighbours {
                self.B.entry(w).or_insert(BTreeSet::new()).insert(v);
            }
        }

        self.stack.pop();
        return f;
    }

    fn unblock(&mut self, u: usize) {
        self.blocked.remove(&u);

        let mut stack: Vec<usize> = self.B.remove(&u).into_iter().flatten().collect();

        while let Some(w) = stack.pop() {
            if self.blocked.remove(&w) {
                stack.extend(self.B.remove(&w).into_iter().flatten());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::centralised::core::Node;

    #[test]
    fn initialisation() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![5]),
            Node::new(5, vec![2]),
        ]);

        let johnson = Johnson::new(graph.clone());

        assert!(johnson.B.is_empty());
        assert!(johnson.blocked.is_empty());
        assert!(johnson.circuits.is_empty());
        assert_eq!(johnson.graph, graph);
        assert_eq!(johnson.n, 5);
        assert_eq!(johnson.s, 2);
        assert!(johnson.stack.is_empty());
        assert!(johnson.subgraph.nodes.is_empty());
    }

    #[test]
    fn detect_circuit_none() {
        let graph = Graph::new(vec![
            Node::new(2, vec![]),
            Node::new(3, vec![]),
            Node::new(5, vec![]),
        ]);

        let circuits = Johnson::new(graph).detect();

        assert!(circuits.is_empty());
    }

    #[test]
    fn detect_circuit_single() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![5]),
            Node::new(5, vec![2]),
        ]);

        let circuits = Johnson::new(graph).detect();

        assert_eq!(circuits, vec![vec![2, 3, 5, 2]]);
    }

    #[test]
    fn detect_circuit_multiple() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![2]),
            Node::new(5, vec![7]),
            Node::new(7, vec![5]),
        ]);

        let circuits = Johnson::new(graph).detect();

        assert_eq!(circuits, vec![vec![2, 3, 2], vec![5, 7, 5]]);
    }

    #[test]
    fn detect_circuit_complex() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![5, 19]),
            Node::new(5, vec![7, 17]),
            Node::new(7, vec![11]),
            Node::new(11, vec![5, 13]),
            Node::new(13, vec![]),
            Node::new(17, vec![7, 13]),
            Node::new(19, vec![2, 17]),
        ]);

        let circuits = Johnson::new(graph).detect();

        assert_eq!(
            circuits,
            vec![vec![2, 3, 19, 2], vec![5, 7, 11, 5], vec![5, 17, 7, 11, 5]],
        );
    }
}
