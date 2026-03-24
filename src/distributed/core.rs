use crate::distributed::tarjan::Tarjan;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Participant {
    id: &'static str,
    pub graph: Graph,
    out: BTreeMap<usize, BTreeSet<u128>>,
}

impl Participant {
    pub fn new(id: &'static str, graph: Graph) -> Self {
        Self {
            id,
            graph,
            out: BTreeMap::new(),
        }
    }

    pub fn compute(
        graph: &Graph,
        queries: Vec<Query>,
    ) -> (Vec<Vec<usize>>, HashMap<&'static str, Vec<Candidate>>) {
        let (components, candidates) = Tarjan::new(graph).detect(queries);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        let components = components.into_iter().filter(|c| c.len() > 1).collect();

        return (components, candidates);
    }

    pub fn receive(&self, queries: Vec<Query>) -> (Vec<Query>, Vec<Query>) {
        return queries.into_iter().partition(|query| {
            self.out
                .get(&query.source)
                .and_then(|tokens| tokens.get(&query.token))
                .is_some()
        });
    }

    pub fn send(
        &mut self,
        candidates: HashMap<&'static str, Vec<Candidate>>,
    ) -> HashMap<&'static str, Vec<Query>> {
        return candidates
            .into_iter()
            .map(|(participant, candidates)| {
                (
                    participant,
                    candidates
                        .into_iter()
                        .map(|candidate| {
                            self.out
                                .entry(candidate.source)
                                .or_insert(BTreeSet::new())
                                .insert(candidate.token);

                            return Query::from(&candidate);
                        })
                        .collect(),
                )
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct Query {
    pub path: Vec<usize>,
    pub source: usize,
    pub token: u128,
}

impl Query {
    pub fn new(source: usize) -> Self {
        Self {
            path: Vec::new(),
            source,
            token: rand::random::<u128>(),
        }
    }
}

impl From<&Candidate> for Query {
    fn from(candidate: &Candidate) -> Self {
        Self {
            path: candidate.path.clone(),
            source: *candidate
                .path
                .last()
                .expect("Candidate path must not be empty"),
            token: candidate.token,
        }
    }
}

#[derive(Debug)]
pub struct Candidate {
    pub path: Vec<usize>,
    pub source: usize,
    pub target: usize,
    pub token: u128,
}

impl Candidate {
    pub fn from(query: &Query, target: usize, path: &[usize]) -> Self {
        Self {
            path: query.path.iter().chain(path.into_iter()).copied().collect(),
            source: query.source,
            target: target,
            token: query.token,
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
