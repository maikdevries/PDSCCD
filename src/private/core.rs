use curve25519_dalek::Scalar;
use std::collections::{HashMap, HashSet};

use crate::private::{
    crypto::{Ciphertext, Plaintext, STTP, Sealed, Unsealed},
    tarjan::{Component, Path, Tarjan},
};

#[derive(Clone)]
pub struct Participant<'a> {
    crypto: &'a STTP,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, Vec<Path>>,
    size: usize,
    tokens: HashMap<NID, Vec<Plaintext>>,
}

impl<'a> Participant<'a> {
    pub fn new(id: PID, size: usize, crypto: &'a STTP, graph: Graph) -> Self {
        Self {
            crypto: crypto,
            graph: graph,
            id: id,
            paths: HashMap::new(),
            size: size,
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
                    size: self.size,
                    target: node,
                    token: self.crypto.encrypt(&message),
                };
            })
            .collect();
    }

    pub fn forward(&self, queries: Vec<Query>) -> HashMap<PID, Vec<Query>> {
        return queries.into_iter().fold(HashMap::new(), |mut map, query| {
            // [NOTE]
            for path in self.paths.get(&query.target).into_iter().flatten() {
                // [NOTE]
                if let Some(size) = query.size.checked_sub(path.nodes.len()) {
                    map.entry(path.participant).or_default().push(Query {
                        from: self.id,
                        path: query.path.iter().chain(&path.nodes).copied().collect(),
                        size: size,
                        target: path.target,
                        token: self.crypto.rerandomise(&query.token),
                    });
                }
            }

            return map;
        });
    }

    pub fn decrypt(&self, queries: Vec<Query>) -> (Vec<Component>, Vec<Query>) {
        // [NOTE]
        let groups: HashMap<usize, Vec<Query>> =
            queries.into_iter().fold(HashMap::new(), |mut map, query| {
                map.entry(query.target).or_default().push(query);
                return map;
            });

        let mut components = Vec::new();
        let mut incomplete = Vec::new();

        for (node, queries) in groups {
            let mut cache = HashMap::new();

            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE]
            let seals = queries
                .into_iter()
                .map(|query| {
                    let seal = Sealed {
                        cipher: query.token * alpha,
                        nonce: rand::random::<u128>(),
                    };

                    cache.insert(seal.nonce, query);
                    return seal;
                })
                .collect();

            // [NOTE]
            let blinds = self
                .tokens
                .get(&node)
                .expect("Target node tokens must be known to decrypt")
                .iter()
                .map(|token| *token * beta)
                .collect();

            let (unsealed, blinds) = self.crypto.unseal(seals, blinds);

            // [NOTE]
            let unsealed: Vec<Unsealed> = unsealed
                .into_iter()
                .map(|unsealed| unsealed * beta)
                .collect();

            // [NOTE]
            let blinds: Vec<Plaintext> = blinds.into_iter().map(|blind| blind * alpha).collect();

            // [NOTE]
            for unseal in unsealed {
                if blinds.contains(&unseal.plain) {
                    components.push(
                        cache
                            .remove(&unseal.nonce)
                            .expect("Unsealed nonce must be known")
                            .path,
                    );
                } else {
                    incomplete.push(
                        cache
                            .remove(&unseal.nonce)
                            .expect("Unsealed nonce must be known"),
                    );
                }
            }
        }

        return (components, incomplete);
    }
}

pub type NID = usize;
pub type PID = &'static str;

// [TODO]
#[derive(Debug)]
pub struct Query {
    pub from: PID,
    pub path: Component,
    pub size: usize,
    pub target: NID,
    pub token: Ciphertext,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone, Copy)]
pub enum Location {
    External(PID),
    Internal,
}
