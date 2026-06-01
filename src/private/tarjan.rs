use std::collections::{HashMap, HashSet};

use crate::private::core::{Graph, Location, NID};

pub type Component = HashSet<NID>;

// [TODO]
#[derive(Clone, Debug)]
pub struct Path {
    pub exit: NID,
    pub nodes: Component,
    pub target: NID,
}

pub struct Tarjan<'a> {
    components: Vec<Component>,
    graph: &'a Graph,
    i: usize,
    lowlink: HashMap<NID, usize>,
    number: HashMap<NID, usize>,
    paths: HashMap<NID, Vec<Path>>,
    stack: Vec<NID>,
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

    pub fn detect(mut self, targets: &HashSet<NID>) -> (Vec<Component>, HashMap<NID, Vec<Path>>) {
        for target in targets {
            if !self.number.contains_key(target) {
                self.strong_connect(*target);
            }
        }

        // [NOTE]
        return (
            self.components,
            self.paths
                .extract_if(|node, _| targets.contains(node))
                .collect(),
        );
    }

    fn strong_connect(&mut self, v: NID) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        for w in self.graph.nodes[&v].neighbours.iter() {
            // [NOTE]
            if let Location::External(_) = self.graph.nodes[w].location {
                self.paths.entry(v).or_default().push(Path {
                    exit: v,
                    nodes: [v].into(),
                    target: *w,
                });

            // [NOTE]
            } else if !self.number.contains_key(w) {
                self.strong_connect(*w);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[w]));

            // [NOTE]
            } else if self.number[w] < self.number[&v] && self.stack.contains(w) {
                self.lowlink.insert(v, self.lowlink[&v].min(self.number[w]));
            }

            // [NOTE]
            if self.paths.contains_key(w) {
                let paths: Vec<Path> = self.paths[w]
                    .iter()
                    .map(|path| Path {
                        exit: path.exit,
                        nodes: [v].iter().chain(&path.nodes).copied().collect(),
                        target: path.target,
                    })
                    .collect();

                self.paths.entry(v).or_default().extend(paths);
            }
        }

        // [NOTE]
        if self.lowlink[&v] == self.number[&v] {
            let mut scc = HashSet::new();

            // [NOTE]
            while let Some(w) = self.stack.pop_if(|w| self.number[w] >= self.number[&v]) {
                scc.insert(w);
            }

            if scc.len() > 1 {
                self.components.push(scc);
            }
        }
    }
}
