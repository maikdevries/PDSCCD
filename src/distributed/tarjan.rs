use crate::distributed::core::{Candidate, Graph, Location, Query};
use std::collections::{HashMap, HashSet};

pub struct Tarjan<'a> {
    components: Vec<HashSet<usize>>,
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
        queries: Vec<Query>,
    ) -> (Vec<HashSet<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        for query in queries {
            self.strong_connect(query.target, &query);
        }

        return (self.components, self.candidates);
    }

    fn strong_connect(&mut self, v: usize, query: &Query) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        for w in self.graph.nodes[&v].neighbours.clone() {
            if let Location::External(participant) = self.graph.nodes[&w].location
                && !self.stack.is_empty()
            {
                self.candidates
                    .entry(participant)
                    .or_insert(Vec::new())
                    .push(Candidate::from(query, w, &self.stack));
            } else if !self.number.contains_key(&w) {
                self.strong_connect(w, query);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[&w]));
            } else if self.number[&w] < self.number[&v] && self.stack.contains(&w) {
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.number[&w]));
            }
        }

        if self.lowlink[&v] == self.number[&v] {
            let mut scc = HashSet::new();

            while let Some(w) = self.stack.pop_if(|w| self.number[w] >= self.number[&v]) {
                scc.insert(w);
            }

            self.components.push(scc);
        }
    }
}
