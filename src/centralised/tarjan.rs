use std::collections::BTreeMap;

use crate::centralised::core::Graph;

pub struct Tarjan<'a> {
    components: Vec<Vec<usize>>,
    graph: &'a Graph,
    i: usize,
    lowlink: BTreeMap<usize, usize>,
    number: BTreeMap<usize, usize>,
    stack: Vec<usize>,
}

impl<'a> Tarjan<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            components: Vec::new(),
            graph,
            i: 0,
            lowlink: BTreeMap::new(),
            number: BTreeMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(mut self) -> Vec<Vec<usize>> {
        for w in self.graph.nodes.keys().copied().collect::<Vec<usize>>() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w);
            }
        }

        return self.components;
    }

    fn strong_connect(&mut self, v: usize) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        for w in self.graph.nodes[&v].neighbours.clone() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[&w]));
            // [PERF] Use sorted data structure to avoid expensive linear search per node
            } else if self.number[&w] < self.number[&v] && self.stack.contains(&w) {
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.number[&w]));
            }
        }

        if self.lowlink[&v] == self.number[&v] {
            let mut scc = Vec::new();

            while let Some(w) = self.stack.pop_if(|w| self.number[w] >= self.number[&v]) {
                scc.push(w);
            }

            self.components.push(scc);
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

        let tarjan = Tarjan::new(&graph);

        assert!(tarjan.components.is_empty());
        assert_eq!(tarjan.graph, &graph);
        assert_eq!(tarjan.i, 0);
        assert!(tarjan.lowlink.is_empty());
        assert!(tarjan.number.is_empty());
        assert!(tarjan.stack.is_empty());
    }

    #[test]
    fn detect_components_disconnected() {
        let graph = Graph::new(vec![
            Node::new(2, vec![]),
            Node::new(3, vec![]),
            Node::new(5, vec![]),
        ]);

        let components = Tarjan::new(&graph).detect();

        assert_eq!(components, vec![vec![2], vec![3], vec![5]]);
    }

    #[test]
    fn detect_components_cycle() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![5]),
            Node::new(5, vec![2]),
        ]);

        let components = Tarjan::new(&graph).detect();

        assert_eq!(components, vec![vec![5, 3, 2]]);
    }

    #[test]
    fn detect_components_unconnected_cycles() {
        let graph = Graph::new(vec![
            Node::new(2, vec![3]),
            Node::new(3, vec![2]),
            Node::new(5, vec![7]),
            Node::new(7, vec![5]),
        ]);

        let components = Tarjan::new(&graph).detect();

        assert_eq!(components, vec![vec![3, 2], vec![7, 5]]);
    }

    #[test]
    fn detect_components_complex() {
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

        let components = Tarjan::new(&graph).detect();

        assert_eq!(
            components,
            vec![vec![13], vec![17, 11, 7, 5], vec![19, 3, 2]]
        );
    }
}
