use crate::distributed::core::{Candidate, Graph, Location};
use std::collections::HashMap;

pub struct Tarjan<'a> {
    components: Vec<Vec<usize>>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    candidates: HashMap<&'static str, Vec<Candidate>>,
    stack: Vec<usize>,
}

impl<'a> Tarjan<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            components: Vec::new(),
            graph,
            i: 0,
            lowlink: HashMap::new(),
            number: HashMap::new(),
            candidates: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(
        mut self,
        candidates: Vec<Candidate>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        for candidate in candidates {
            if !self.number.contains_key(&candidate.source) {
                self.strong_connect(candidate.source, &candidate);
            }
        }

        return (self.components, self.candidates);
    }

    fn strong_connect(&mut self, v: usize, candidate: &Candidate) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        for w in self.graph.nodes[&v].neighbours.clone() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w, candidate);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[&w]));
            // [PERF] Use sorted data structure to avoid expensive linear search per node
            } else if self.number[&w] < self.number[&v] && self.stack.contains(&w) {
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.number[&w]));
            }
        }

        if let Location::External(participant) = self.graph.nodes[&v].location {
            // [NOTE] Path consists of internal nodes and should not include source and target nodes
            let path = Vec::from(self.stack.get(1..self.stack.len() - 1).unwrap_or_default());

            if !path.is_empty() {
                self.candidates
                    .entry(participant)
                    .or_insert(Vec::new())
                    .push(candidate.with(v, path));
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
