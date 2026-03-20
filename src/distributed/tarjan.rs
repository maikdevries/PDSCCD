use crate::distributed::core::{Graph, Location};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Partial {
    path: Vec<usize>,
    sink: usize,
    source: usize,
}

impl Partial {
    fn new(source: usize, sink: usize, path: Vec<usize>) -> Self {
        Self { path, sink, source }
    }
}

pub struct Tarjan<'a> {
    components: Vec<Vec<usize>>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    partials: HashMap<&'static str, Vec<Partial>>,
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
            partials: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(
        mut self,
        roots: Vec<&usize>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Partial>>) {
        for w in roots {
            if !self.number.contains_key(w) {
                self.strong_connect(*w, *w);
            }
        }

        return (self.components, self.partials);
    }

    fn strong_connect(&mut self, v: usize, root: usize) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        // [PERF] Use reference to avoid expensive clone of neighbours
        for w in self.graph.nodes[&v].neighbours.clone() {
            if !self.number.contains_key(&w) {
                self.strong_connect(w, root);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[&w]));
            // [PERF] Use sorted data structure to avoid expensive linear search per node
            } else if self.number[&w] < self.number[&v] && self.stack.contains(&w) {
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.number[&w]));
            }
        }

        if let Location::External(participant) = self.graph.nodes[&v].location {
            self.partials
                .entry(participant)
                .or_insert(Vec::new())
                .push(Partial::new(
                    root,
                    v,
                    Vec::from(self.stack.get(1..self.stack.len() - 1).unwrap_or_default()),
                ));
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
