use curve25519_dalek::Scalar;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::private::{
    crypto::{Ciphertext, Elliptic, Sealed},
    tarjan::{Component, Path, Tarjan},
};

#[derive(Clone)]
pub struct Participant {
    capacity: usize,
    crypto: Arc<Elliptic>,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, Vec<Path>>,
    tokens: HashMap<NID, Scalar>,
}

impl Participant {
    pub fn new(id: PID, graph: Graph, crypto: Arc<Elliptic>, capacity: usize) -> Self {
        Self {
            capacity: capacity,
            crypto: crypto,
            graph: graph,
            id: id,
            paths: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn partition(&self, messages: Vec<Message>) -> (Vec<Message>, Vec<Message>) {
        // [NOTE] Partition received messages based on whether target node has a token stored
        return messages
            .into_iter()
            .partition(|message| self.tokens.contains_key(&message.target));
    }

    pub fn compute(&mut self, targets: &HashSet<NID>) -> Vec<Component> {
        let (components, paths) = Tarjan::new(&self.graph).tarjan(targets);

        for (k, v) in paths {
            self.paths.entry(k).or_default().extend(v);
        }

        return components;
    }

    pub fn compose(&mut self, nodes: HashSet<NID>) -> Vec<Message> {
        // [NOTE] Compose new mesages for each unseen target node
        return nodes
            .into_iter()
            .map(|node| {
                let token = Scalar::random(&mut rand::rng());
                self.tokens.insert(node, token);

                return Message {
                    capacity: self.capacity,
                    nodes: Vec::new(),
                    target: node,
                    token: self.crypto.encrypt(&Elliptic::encode(token)),
                };
            })
            .collect();
    }

    pub fn forward(&self, messages: Vec<Message>) -> HashMap<PID, Vec<Message>> {
        return messages
            .into_iter()
            .fold(HashMap::new(), |mut map, message| {
                // [NOTE] Forward message copy along detected paths
                for path in self.paths.get(&message.target).into_iter().flatten() {
                    if let Some(capacity) = message.capacity.checked_sub(path.nodes.len())
                        && let Location::External(participant) =
                            self.graph.nodes[&path.target].location
                    {
                        // [NOTE] Re-randomise message token to sever linkability
                        let token = self.crypto.rerandomise(&message.token);

                        map.entry(participant).or_default().push(Message {
                            capacity: capacity,
                            nodes: message
                                .nodes
                                .iter()
                                .map(|c| self.crypto.rerandomise(c))
                                .chain(path.nodes.iter().map(|n| token * Scalar::from(*n)))
                                .collect(),
                            target: path.target,
                            token: token,
                        });
                    }
                }

                return map;
            });
    }

    pub fn recognise(&self, messages: Vec<Message>) -> (HashMap<NID, Component>, Vec<Message>) {
        // [NOTE] Partition messages into groups based on target node
        let groups: HashMap<NID, Vec<Message>> =
            messages
                .into_iter()
                .fold(HashMap::new(), |mut map, message| {
                    map.entry(message.target).or_default().push(message);
                    return map;
                });

        let mut components: HashMap<NID, Component> = HashMap::new();
        let mut incomplete = Vec::new();

        for (node, queries) in groups {
            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            // [NOTE] Blind each received message token and store message in cache
            let mut cache = HashMap::with_capacity(queries.len());
            let seals: Vec<Sealed> = queries
                .into_iter()
                .map(|message| {
                    let seal = Sealed {
                        nonce: rand::random::<u128>(),
                        token: message.token * alpha,
                    };

                    cache.insert(seal.nonce, message);
                    return seal;
                })
                .collect();

            // [NOTE] Blind the stored token with separate blinding scalar
            let token = self.tokens.get(&node).unwrap();
            let blind = Elliptic::encode(*token) * beta;

            // [NOTE] Interact with STTP to decrypt message tokens
            let (unsealed, blind) = self.crypto.unseal(seals, blind);
            let blind = blind * alpha;

            // [NOTE] For each decrypted message token, check whether it's equivalent to stored token
            for unseal in unsealed {
                let message = cache.remove(&unseal.nonce).unwrap();

                if unseal.token * beta == blind {
                    let gamma = Scalar::random(&mut rand::rng());

                    // [NOTE] For each recognised token, interact with the STTP to decrypt node identifiers
                    let component: Component = message
                        .nodes
                        .into_iter()
                        .map(|c| {
                            self.crypto
                                .recover(
                                    &(self.crypto.decrypt(&(c * gamma))
                                        * gamma.invert()
                                        * token.invert()),
                                )
                                .unwrap()
                        })
                        .collect();

                    components.entry(node).or_default().extend(component);
                } else {
                    incomplete.push(message);
                }
            }
        }

        return (components, incomplete);
    }
}

pub type NID = u32;
pub type PID = &'static str;

#[derive(Debug)]
pub struct Message {
    pub capacity: usize,
    pub nodes: Vec<Ciphertext>,
    pub target: NID,
    pub token: Ciphertext,
}

#[derive(Clone)]
pub struct Graph {
    pub nodes: HashMap<NID, Node>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>) -> Self {
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
    pub fn new(id: NID, location: Location, neighbours: Vec<NID>) -> Self {
        Self {
            id: id,
            location: location,
            neighbours: neighbours,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Location {
    External(PID),
    Internal,
}
