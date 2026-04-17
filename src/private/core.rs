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
    // nonces: HashMap<u128, u128>,
    paths: HashMap<usize, HashMap<PID, Vec<Path>>>,
    tokens: HashMap<usize, Vec<Plaintext>>,
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

    pub fn query(&self, queries: Vec<Query>) -> (Vec<Query>, Vec<Query>) {
        // [NOTE]
        // [TODO] Combine queries with same target and token into single query (?)
        return queries
            .into_iter()
            .partition(|query| self.tokens.get(&query.target).is_some());
    }

    pub fn detect(&mut self, targets: Vec<usize>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).detect(targets);

        // [NOTE]
        self.paths.extend(paths);

        // [NOTE] Filter out trivial components (i.e. consist of a single node)
        return components.into_iter().filter(|c| c.len() > 1).collect();
    }

    pub fn register(&mut self, queries: Vec<Query>) -> Vec<Query> {
        return queries
            .into_iter()
            .flat_map(|query| {
                let token = Plaintext::from(rand::random::<u128>());
                self.tokens.entry(query.target).or_default().push(token);

                [
                    Query {
                        from: self.id,
                        path: HashSet::new(),
                        target: query.target,
                        token: self.crypto.encrypt(&token),
                    },
                    query,
                ]
            })
            // [TODO] Filter out initial Query
            .filter(|query| !query.from.is_empty())
            .collect();
    }

    pub fn forward(&self, queries: Vec<Query>) -> HashMap<PID, Vec<Query>> {
        // [NOTE]
        // [TODO] Combine query with same target and token into single query (?)
        // [TODO] Do not send queries straight back to originating participant
        // return queries
        //     .into_iter()
        //     .flat_map(|query| {
        //         return self
        //             .paths
        //             .get(&query.target)
        //             .expect("Exit participant(s) must be known to forward")
        //             .iter()
        //             .map(move |(participant, paths)| {
        //                 (
        //                     *participant,
        //                     paths
        //                         .iter()
        //                         .map(|path| Query {
        //                             from: self.id,
        //                             path: query.path.iter().chain(&path.nodes).copied().collect(),
        //                             target: path.target,
        //                             token: self.crypto.rerandomise(&query.token),
        //                         })
        //                         .collect(),
        //                 )
        //             });
        //     })
        //     .collect();

        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            let paths = self
                .paths
                .get(&query.target)
                .expect("Exit participant(s) must be known to forward");

            for (participant, paths) in paths {
                map.entry(participant)
                    .or_default()
                    .extend(paths.iter().map(|path| Query {
                        from: self.id,
                        path: query.path.iter().chain(&path.nodes).copied().collect(),
                        target: path.target,
                        token: self.crypto.rerandomise(&query.token),
                    }));
            }

            return map;
        });
    }

    pub fn request(&mut self, queries: Vec<Query>) -> HashMap<PID, Vec<Request>> {
        // [NOTE]
        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            // [BUG] Other participant might be same as query origin
            let other = self
                .paths
                .get(&query.target)
                .expect("Query target must be known to decrypt")
                .keys()
                .choose(&mut rand::rng())
                .expect("Exit participant must be known to decrypt");

            // [BUG] All decryption participants will receive same nonce
            let nonce = rand::random::<u128>();

            // [NOTE]
            // [TODO] Unnecessary to send full ciphertext
            map.entry(query.from).or_default().push(Request {
                from: self.id,
                nonce: nonce,
                token: query.token,
            });
            map.entry(other).or_default().push(Request {
                from: self.id,
                nonce: nonce,
                token: query.token,
            });

            self.candidates.insert(
                nonce,
                Candidate {
                    partials: [self.crypto.decrypt(&query.token)].into(),
                    query: query,
                },
            );

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
        for response in responses {
            self.candidates
                .get_mut(&response.nonce)
                .expect("Response nonce must be known to combine partials")
                .partials
                .push(response.partial);
        }

        let candidates: Vec<Candidate> = self
            .candidates
            .extract_if(|_, candidate| candidate.partials.len() >= 3)
            .map(|(_, candidate)| candidate)
            .collect();

        let mut components = Vec::new();
        let mut queries = Vec::new();

        for candidate in candidates {
            let plain = Threshold::combine(&candidate.partials, candidate.query.token);
            let tokens = self
                .tokens
                .get(&candidate.query.target)
                .expect("Candidate target must be known to combine");

            if tokens.contains(&plain) {
                components.push(candidate.query.path);
            } else {
                queries.push(candidate.query);
            }
        }

        return (components, queries);
    }
}

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
    pub target: usize,
    pub token: Ciphertext,
}

// [TODO]
#[derive(Debug)]
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
    pub nodes: HashMap<usize, Node>,
}

impl Graph {
    pub fn new<const N: usize>(nodes: [Node; N]) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| (n.id, n)).collect(),
        }
    }
}

pub struct Node {
    id: usize,
    pub location: Location,
    pub neighbours: HashSet<usize>,
}

impl Node {
    pub fn new<const N: usize>(id: usize, location: Location, neighbours: [usize; N]) -> Self {
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
