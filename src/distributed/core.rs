use crate::distributed::tarjan::Tarjan;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Participant {
    id: &'static str,
    pub graph: Graph,
    out: HashMap<usize, HashMap<u128, Candidate>>,
}

impl Participant {
    pub fn new(id: &'static str, graph: Graph) -> Self {
        Self {
            id,
            graph,
            out: HashMap::new(),
        }
    }

    pub fn compute(
        graph: &Graph,
        candidates: Vec<Candidate>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        let (components, candidates) = Tarjan::new(graph).detect(candidates);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        let components = components.into_iter().filter(|c| c.len() > 1).collect();

        return (components, candidates);
    }

    pub fn receive(&self, mut candidates: Vec<Candidate>) -> (Vec<Candidate>, Vec<Candidate>) {
        let mut resolved = Vec::new();

        candidates.retain(|c| {
            if let Some(candidate) = self
                .out
                .get(&c.source)
                .and_then(|tokens| tokens.get(&c.token))
            {
                resolved.push(Candidate {
                    path: c.path.clone(),
                    sink: Some(c.source),
                    source: candidate.source,
                    token: candidate.token,
                });
                return false;
            } else {
                return true;
            }
        });

        return (resolved, candidates);
    }

    pub fn send(
        &mut self,
        candidates: HashMap<&'static str, Vec<Candidate>>,
    ) -> HashMap<&'static str, Vec<Candidate>> {
        return candidates
            .into_iter()
            .map(|(participant, candidates)| {
                (
                    participant,
                    candidates
                        .into_iter()
                        .map(|c| {
                            let candidate = Candidate {
                                path: c.path.clone(),
                                source: *c.path.last().expect("Candidate path must not be empty"),
                                sink: None,
                                token: c.token,
                            };

                            self.out
                                .entry(c.source)
                                .or_insert(HashMap::new())
                                .insert(c.token, c);

                            return candidate;
                        })
                        .collect(),
                )
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct Candidate {
    pub path: Vec<usize>,
    pub sink: Option<usize>,
    pub source: usize,
    pub token: u128,
}

impl Candidate {
    pub fn new(source: usize) -> Self {
        Self {
            path: Vec::new(),
            sink: None,
            source,
            token: rand::random::<u128>(),
        }
    }

    pub fn with(&self, sink: usize, path: Vec<usize>) -> Self {
        Self {
            path: self.path.iter().cloned().chain(path).collect(),
            sink: Some(sink),
            source: self.source,
            token: self.token,
        }
    }
}

pub struct Graph {
    pub nodes: BTreeMap<usize, Node>,
}

impl Graph {
    pub fn new(adjacency: Vec<Node>) -> Self {
        Self {
            nodes: adjacency.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    pub id: usize,
    pub location: Location,
    pub neighbours: BTreeSet<usize>,
}

impl Node {
    pub fn new(id: usize, location: Location, neighbours: Vec<usize>) -> Self {
        Self {
            id,
            location,
            neighbours: BTreeSet::from_iter(neighbours),
        }
    }
}

pub enum Location {
    External(&'static str),
    Internal,
}
