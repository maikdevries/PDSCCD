use std::collections::{HashMap, HashSet};

use crate::private::core::{Graph, Location, PID};

pub type Component = HashSet<usize>;

// [TODO]
#[derive(Debug)]
pub struct Path {
    pub nodes: Component,
    pub target: usize,
}

pub struct Tarjan<'a> {
    components: Vec<Component>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<usize, usize>,
    number: HashMap<usize, usize>,
    paths: HashMap<PID, Vec<Path>>,
    stack: Vec<usize>,
}

impl<'a> Tarjan<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            components: Vec::new(),
            graph: graph,
            i: 0,
            lowlink: HashMap::new(),
            number: HashMap::new(),
            paths: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn detect(
        mut self,
        targets: Vec<usize>,
    ) -> (Vec<Component>, HashMap<usize, HashMap<PID, Vec<Path>>>) {
        let mut paths = HashMap::new();

        // [TODO] Targets that connect to previously seen nodes should inherit paths
        for target in targets {
            // [BUG] Check whether target has been computed already
            if paths.contains_key(&target) {
                continue;
            }

            self.strong_connect(target);
            paths.insert(target, self.paths.drain().collect());
        }

        return (self.components, paths);
    }

    fn strong_connect(&mut self, v: usize) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        for w in self.graph.nodes[&v].neighbours.iter() {
            if let Location::External(participant) = self.graph.nodes[w].location
                && !self.stack.is_empty()
            {
                self.paths.entry(participant).or_default().push(Path {
                    nodes: self.stack.iter().copied().collect(),
                    target: *w,
                });
            } else if !self.number.contains_key(w) {
                self.strong_connect(*w);
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
