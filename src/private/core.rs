use curve25519_dalek::Scalar;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::private::{
    crypto::{Ciphertext, Crypto, Sealed},
    tarjan::{Component, Tarjan},
};

#[derive(Clone)]
pub struct Participant {
    capacity: usize,
    crypto: Arc<Crypto>,
    pub graph: Graph,
    pub id: PID,
    paths: HashMap<NID, HashMap<NID, HashMap<NID, Component>>>,
    tokens: HashMap<NID, Scalar>,
}

impl Participant {
    pub fn new(id: PID, graph: Graph, crypto: Arc<Crypto>, capacity: usize) -> Self {
        Self {
            capacity: capacity,
            crypto: crypto,
            graph: graph,
            id: id,
            paths: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn receive(&self, messages: Vec<Message>) -> (Vec<Message>, Vec<Message>) {
        // [NOTE]
        return messages
            .into_iter()
            .partition(|message| self.tokens.contains_key(&message.target));
    }

    pub fn detect(&mut self, targets: &HashSet<NID>) -> (Vec<Component>, u128) {
        if targets.is_empty() {
            return (Vec::new(), 0);
        }

        let (components, paths) = Tarjan::new(&self.graph).detect(targets);
        let mut space = 0;

        // [NOTE]
        for (k, v) in paths {
            for p in v {
                space += (std::mem::size_of_val(&p.exit)
                    + std::mem::size_of_val(&p.target)
                    + (p.nodes.len() * std::mem::size_of::<NID>()))
                    as u128;

                self.paths
                    .entry(k)
                    .or_default()
                    .entry(p.exit)
                    .or_default()
                    .entry(p.target)
                    .or_default()
                    .extend(p.nodes);
            }
        }

        return (components, space);
    }

    pub fn register(&mut self, nodes: HashSet<NID>) -> (Vec<Message>, u128) {
        let mut space: u128 = 0;

        // [NOTE]
        return (
            nodes
                .into_iter()
                .map(|node| {
                    let token = Scalar::random(&mut rand::rng());
                    self.tokens.insert(node, token);

                    space += std::mem::size_of_val(&token) as u128;

                    return Message {
                        capacity: self.capacity,
                        nodes: Vec::new(),
                        target: node,
                        token: self.crypto.encrypt(&Crypto::encode(token)),
                    };
                })
                .collect(),
            space,
        );
    }

    pub fn forward(&self, messages: Vec<Message>) -> HashMap<PID, Vec<Message>> {
        return messages
            .into_iter()
            .fold(HashMap::new(), |mut map, message| {
                // [NOTE]
                for (_, targets) in self.paths.get(&message.target).into_iter().flatten() {
                    for (&target, nodes) in targets {
                        // [NOTE]
                        if let Some(capacity) = message.capacity.checked_sub(nodes.len())
                            && let Location::External(participant) =
                                self.graph.nodes[&target].location
                        {
                            // [NOTE]
                            let token = self.crypto.rerandomise(&message.token);

                            map.entry(participant).or_default().push(Message {
                                capacity: capacity,
                                nodes: message
                                    .nodes
                                    .iter()
                                    .map(|c| self.crypto.rerandomise(c))
                                    .chain(nodes.iter().map(|n| token * Scalar::from(*n)))
                                    .collect(),
                                target: target,
                                token: token,
                            });
                        }
                    }
                }

                return map;
            });
    }

    pub fn decrypt(
        &self,
        messages: Vec<Message>,
    ) -> (HashMap<NID, Component>, Vec<Message>, u128, u128) {
        // [NOTE]
        let groups: HashMap<NID, Vec<Message>> =
            messages
                .into_iter()
                .fold(HashMap::new(), |mut map, message| {
                    map.entry(message.target).or_default().push(message);
                    return map;
                });

        let mut components: HashMap<NID, Component> = HashMap::new();
        let mut incomplete = Vec::new();

        let mut communication: u128 = 0;
        let mut space: u128 = 0;

        for (node, queries) in groups {
            let alpha = Scalar::random(&mut rand::rng());
            let beta = Scalar::random(&mut rand::rng());

            space += (std::mem::size_of_val(&alpha) + std::mem::size_of_val(&beta)) as u128;

            // [NOTE]
            let mut cache = HashMap::with_capacity(queries.len());
            let seals: Vec<Sealed> = queries
                .into_iter()
                .map(|message| {
                    let seal = Sealed {
                        nonce: rand::random::<u128>(),
                        token: message.token * alpha,
                    };

                    space += std::mem::size_of_val(&seal.nonce) as u128;

                    cache.insert(seal.nonce, message);
                    return seal;
                })
                .collect();

            // [NOTE]
            let token = self.tokens.get(&node).unwrap();
            let blind = Crypto::encode(*token) * beta;

            let size = (std::mem::size_of_val(&seals[..]) + std::mem::size_of_val(&blind)) as u128;
            communication += size;
            space += size;

            // [NOTE]
            let (unsealed, blind) = self.crypto.unseal(seals, blind);
            let blind = blind * alpha;

            // [NOTE]
            for unseal in unsealed {
                let message = cache.remove(&unseal.nonce).unwrap();

                if unseal.token * beta == blind {
                    let gamma = Scalar::random(&mut rand::rng());

                    let size = std::mem::size_of_val(&message.nodes[..]) as u128;
                    communication += size;
                    space += size + std::mem::size_of_val(&gamma) as u128;

                    // [NOTE]
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

                    space += (component.len() * std::mem::size_of::<NID>()) as u128;
                    components.entry(node).or_default().extend(component);
                } else {
                    incomplete.push(message);
                }
            }
        }

        return (components, incomplete, communication, space);
    }
}

pub type NID = u32;
pub type PID = &'static str;

// [TODO]
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
