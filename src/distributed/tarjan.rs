use crate::distributed::core::{Candidate, Component, Graph, Location, PID, Query};
use std::collections::{HashMap, HashSet};

pub struct Tarjan<'a> {
    candidates: HashMap<PID, Vec<Candidate>>,
    components: Vec<Component>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    stack: Vec<usize>,
}

impl<'a> Tarjan<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            candidates: HashMap::new(),
            components: Vec::new(),
            graph,
            i: 0,
            lowlink: HashMap::new(),
            number: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(mut self, queries: Vec<Query>) -> (Vec<Component>, HashMap<PID, Vec<Candidate>>) {
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

        for w in self.graph.nodes[&v].neighbours.iter() {
            if let Location::External(participant) = self.graph.nodes[w].location
                && !self.stack.is_empty()
            {
                self.candidates
                    .entry(participant)
                    .or_insert(Vec::new())
                    .push(Candidate::from(query, *w, &self.stack));
            } else if !self.number.contains_key(w) {
                self.strong_connect(*w, query);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[w]));
            } else if self.number[w] < self.number[&v] && self.stack.contains(w) {
                self.lowlink.insert(v, self.lowlink[&v].min(self.number[w]));
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
