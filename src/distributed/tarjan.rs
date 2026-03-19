use crate::distributed::core::{Graph, Location};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Query {
    path: Vec<usize>,
    sinks: HashSet<usize>,
    sources: HashSet<usize>,
}

impl Query {
    fn new() -> Self {
        Self {
            path: Vec::new(),
            sinks: HashSet::new(),
            sources: HashSet::new(),
        }
    }
}

pub struct Tarjan<'a> {
    components: Vec<Vec<usize>>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    queries: Vec<Query>,
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
            queries: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(mut self, roots: Vec<&usize>) -> Vec<Vec<usize>> {
        for w in roots {
            let mut query = Query::new();
            query.sources.insert(*w);

            if !self.number.contains_key(w) {
                self.strong_connect(*w, &mut query);
            }

            self.queries.push(query);
        }

        return self.components;
    }

    fn strong_connect(&mut self, v: usize, query: &mut Query) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        for w in self.graph.nodes[&v].neighbours.clone() {
            match self.graph.nodes[&w].location {
                Location::External(_) => {
                    query.sinks.insert(w);
                }
                Location::Internal => {
                    query.path.push(w);
                }
            }

            if !self.number.contains_key(&w) {
                self.strong_connect(w, query);
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
