use std::collections::HashMap;

use crate::graph::core::Graph;

pub struct Tarjan<'a> {
    components: Vec<Vec<usize>>,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    stack: Vec<usize>,
    graph: &'a Graph,
}

impl<'a> Tarjan<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            components: Vec::new(),
            graph,
            i: 0,
            lowlink: HashMap::new(),
            number: HashMap::new(),
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
