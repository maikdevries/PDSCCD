use crate::distributed::tarjan::Tarjan;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct Participant {
    pub graph: Graph,
    out: BTreeMap<usize, BTreeSet<u128>>,
}

impl Participant {
    pub fn new(graph: Graph) -> Self {
        Self {
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
                .get(&query.target)
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
                        .inspect(|candidate| {
                            self.out
                                .entry(candidate.source)
                                .or_insert(BTreeSet::new())
                                .insert(candidate.token);
                        })
                        // [NOTE] Combine candidates with same target and token into single query
                        .fold(HashMap::new(), |mut map, candidate| {
                            map.entry((candidate.target, candidate.token))
                                .and_modify(|query: &mut Query| {
                                    query.path.extend(candidate.path.clone())
                                })
                                .or_insert_with(|| Query::from(&candidate));

                            return map;
                        })
                        .into_values()
                        .collect(),
                )
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct Query {
    pub path: Vec<usize>,
    pub target: usize,
    pub token: u128,
}

impl Query {
    pub fn new(target: usize, token: u128) -> Self {
        Self {
            path: Vec::new(),
            target,
            token,
        }
    }
}

impl From<&Candidate> for Query {
    fn from(candidate: &Candidate) -> Self {
        Self {
            path: candidate.path.clone(),
            target: candidate.target,
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
    pub fn from(query: &Query, target: usize, path: &Vec<usize>) -> Self {
        Self {
            path: query.path.iter().chain(path).copied().collect(),
            source: query.target,
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
