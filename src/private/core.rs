use rand::seq::IteratorRandom;
use std::collections::{HashMap, HashSet};

use crate::private::{
    crypto::{Ciphertext, Partial, Plaintext, Threshold},
    tarjan::{Component, Path, Tarjan},
};

pub struct Participant {
    candidates: HashMap<u128, Candidate>,
    crypto: Threshold,
    pub graph: Graph,
    id: PID,
    paths: HashMap<NID, Vec<Path>>,
    tokens: HashMap<NID, Vec<Plaintext>>,
}

impl Participant {
    pub fn new(id: PID, share: Threshold, graph: Graph) -> Self {
        Self {
            candidates: HashMap::new(),
            crypto: share,
            graph: graph,
            id: id,
            paths: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn receive(&self, queries: Vec<Query>) -> (Vec<Query>, Vec<Query>) {
        // [NOTE]
        return queries
            .into_iter()
            .partition(|query| self.tokens.get(&query.target).is_some());
    }

    pub fn detect(&mut self, targets: HashSet<NID>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).detect(targets);

        // [NOTE]
        self.paths.extend(paths);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        return components.into_iter().filter(|c| c.len() > 1).collect();
    }

    pub fn register(&mut self, nodes: HashSet<NID>) -> Vec<Query> {
        // [NOTE]
        return nodes
            .into_iter()
            .map(|node| {
                let message = Plaintext::from(rand::random::<u128>());
                self.tokens.entry(node).or_default().push(message);

                return Query {
                    from: self.id,
                    path: [].into(),
                    target: node,
                    token: self.crypto.encrypt(&message),
                };
            })
            .collect();
    }

    pub fn forward(&self, queries: Vec<Query>) -> HashMap<PID, Vec<Query>> {
        // [TODO] Do not send queries straight back to query origin
        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            // [NOTE]
            for path in self.paths.get(&query.target).into_iter().flatten() {
                map.entry(path.participant).or_default().push(Query {
                    from: self.id,
                    path: query.path.iter().chain(&path.nodes).copied().collect(),
                    target: path.target,
                    token: self.crypto.rerandomise(&query.token),
                });
            }

            return map;
        });
    }

    pub fn request(&mut self, queries: Vec<Query>) -> HashMap<PID, Vec<Request>> {
        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            // [NOTE]
            // [BUG] Other participant might be same as query origin
            if let Some(other) = self
                .paths
                .get(&query.target)
                .into_iter()
                .flatten()
                .choose(&mut rand::rng())
            {
                // [BUG] All decryption participants will receive same nonce
                // [BUG] Query token is not re-randomised - linkable by query origin
                // [TODO] Unnecessary to send complete ciphertext - only requires randomness
                let request = Request {
                    from: self.id,
                    nonce: rand::random::<u128>(),
                    token: query.token,
                };

                // [NOTE]
                map.entry(query.from).or_default().push(request);
                map.entry(other.participant).or_default().push(request);

                self.candidates.insert(
                    request.nonce,
                    Candidate {
                        partials: [self.crypto.decrypt(&query.token)].into(),
                        query: query,
                    },
                );
            }

            return map;
        });
    }

    pub fn decrypt(&self, requests: Vec<Request>) -> HashMap<PID, Vec<Response>> {
        return requests
            .into_iter()
            .fold(HashMap::new(), |mut map, request| {
                map.entry(request.from).or_default().push(Response {
                    nonce: request.nonce,
                    partial: self.crypto.decrypt(&request.token),
                });

                return map;
            });
    }

    pub fn combine(&mut self, responses: Vec<Response>) -> (Vec<Component>, Vec<Query>) {
        // [NOTE]
        return responses
            .into_iter()
            .filter_map(|response| {
                // [NOTE]
                let candidate = self.candidates.get_mut(&response.nonce)?;
                candidate.partials.push(response.partial);

                // [NOTE]
                if candidate.partials.len() < 3 {
                    return None;
                }

                // [NOTE]
                return Some((
                    self.tokens.get(&candidate.query.target)?,
                    self.candidates.remove(&response.nonce)?,
                ));
            })
            // [NOTE]
            .fold(
                (Vec::new(), Vec::new()),
                |(mut components, mut queries), (tokens, candidate)| {
                    let plain = Threshold::combine(candidate.partials, &candidate.query.token);

                    if tokens.contains(&plain) {
                        components.push(candidate.query.path);
                    } else {
                        queries.push(candidate.query);
                    }

                    return (components, queries);
                },
            );
    }
}

pub type NID = usize;
pub type PID = &'static str;

struct Candidate {
    partials: Vec<Partial>,
    query: Query,
}

// [TODO]
#[derive(Debug)]
pub struct Query {
    pub from: PID,
    pub path: Component,
    pub target: NID,
    pub token: Ciphertext,
}

// [TODO]
#[derive(Clone, Copy, Debug)]
pub struct Request {
    from: PID,
    nonce: u128,
    token: Ciphertext,
}

// [TODO]
#[derive(Debug)]
pub struct Response {
    nonce: u128,
    partial: Partial,
}

pub struct Graph {
    pub nodes: HashMap<NID, Node>,
}

impl Graph {
    pub fn new<const N: usize>(nodes: [Node; N]) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    id: NID,
    pub location: Location,
    pub neighbours: Vec<NID>,
}

impl Node {
    pub fn new<const N: usize>(id: NID, location: Location, neighbours: [NID; N]) -> Self {
        Self {
            id,
            location,
            neighbours: neighbours.into(),
        }
    }
}

pub enum Location {
    External(PID),
    Internal,
}
