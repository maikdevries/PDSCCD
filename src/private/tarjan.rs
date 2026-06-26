use std::collections::{HashMap, HashSet};

use crate::private::core::{Graph, Location, NID};

pub type Component = HashSet<NID>;

#[derive(Clone)]
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

    pub fn tarjan(mut self, targets: &HashSet<NID>) -> (Vec<Component>, HashMap<NID, Vec<Path>>) {
        for target in targets {
            if !self.number.contains_key(target) {
                self.detect(*target);
            }
        }

        // [NOTE] Return detected paths for original target nodes only
        return (
            self.components,
            self.paths
                .extract_if(|node, _| targets.contains(node))
                .collect(),
        );
    }

    fn detect(&mut self, v: NID) {
        self.i += 1;
        self.lowlink.insert(v, self.i);
        self.number.insert(v, self.i);

        self.stack.push(v);

        for w in self.graph.nodes[&v].neighbours.iter() {
            // [NOTE] Construct new path toward external node
            if let Location::External(_) = self.graph.nodes[w].location {
                self.paths.entry(v).or_default().push(Path {
                    exit: v,
                    nodes: [v].into(),
                    target: *w,
                });

            // [NOTE] Recursively detect SCCs & paths for unvisited neighbours
            } else if !self.number.contains_key(w) {
                self.detect(*w);
                self.lowlink
                    .insert(v, self.lowlink[&v].min(self.lowlink[w]));

            // [NOTE] Update current lowlink if neighbour has already been visited
            } else if self.number[w] < self.number[&v] && self.stack.contains(w) {
                self.lowlink.insert(v, self.lowlink[&v].min(self.number[w]));
            }

            // [NOTE] Incremental path construction during backtracking
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

        // [NOTE] Establish new SCC if current node is identified as root node
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
