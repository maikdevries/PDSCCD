use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::private::{
    core::{Location, Message, NID, PID, Participant},
    crypto::Ciphertext,
    tarjan::Component,
};

// ---

#[cfg(debug_assertions)]
macro_rules! debug_println {
    ($( $args:expr ), *) => { println!($( $args ), *) }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_println {
    ($( $args:expr ), *) => {
        ()
    };
}

// ---

pub struct Protocol {
    participants: HashMap<PID, Participant>,
}

impl Protocol {
    pub fn new<const N: usize>(participants: [Participant; N]) -> Self {
        Self {
            participants: participants.into_iter().map(|p| (p.id, p)).collect(),
        }
    }

    pub fn run(self, initiator: PID) -> HashMap<PID, Vec<Component>> {
        let participant = self
            .participants
            .get(initiator)
            .expect("Particpant must have known ID");

        // [NOTE]
        let jobs = Protocol::seed(participant);
        let mut pending = jobs.len();

        // [NOTE]
        let mut results = HashMap::new();

        // [NOTE]
        thread::scope(|scope| {
            let mut channels = HashMap::new();
            let (sender, receiver) = mpsc::channel();

            // [NOTE]
            let (r_sender, r_receiver) = mpsc::channel();

            // [NOTE]
            for (id, participant) in self.participants {
                let (tx, rx) = mpsc::channel();
                channels.insert(id, tx);

                // [NOTE]
                let tx = sender.clone();
                let rtx = r_sender.clone();
                scope.spawn(move || Protocol::work(participant, rx, tx, rtx));
            }

            // [NOTE]
            sender.send(jobs).unwrap();

            // [NOTE]
            while pending > 0 {
                // [NOTE]
                let jobs = receiver.recv().unwrap();

                for (id, queries) in jobs {
                    channels[id].send(queries).unwrap();
                    pending += 1;
                }

                pending -= 1;
            }

            pending = channels.len();
            drop(channels);

            while pending > 0
                && let Ok((id, components)) = r_receiver.recv()
            {
                results.insert(id, components);
                pending -= 1;
            }
        });

        return results;
    }

    fn seed(participant: &Participant) -> HashMap<PID, Vec<Message>> {
        return [(
            participant.id,
            participant
                .graph
                .nodes
                .values()
                // [NOTE]
                .fold(HashSet::new(), |mut set, node| {
                    if matches!(node.location, Location::External(_)) {
                        set.extend(&node.neighbours);
                    }

                    return set;
                })
                .into_iter()
                .map(|node| Message {
                    capacity: 0,
                    nodes: Vec::new(),
                    source: node,
                    target: node,
                    token: Ciphertext::default(),
                })
                .collect(),
        )]
        .into();
    }

    fn work(
        mut participant: Participant,
        receiver: Receiver<Vec<Message>>,
        sender: Sender<HashMap<PID, Vec<Message>>>,
        results: Sender<(PID, Vec<Component>)>,
    ) {
        let mut components: Vec<Component> = Vec::new();

        // [NOTE]
        for queries in receiver {
            debug_println!();
            debug_println!("--- PARTICIPANT {} START ---", participant.id);

            // [NOTE]
            let (known, unknown) = participant.receive(queries);
            debug_println!("[{}] - Known: {known:?}", participant.id);
            debug_println!("[{}] - Unknown: {unknown:?}", participant.id);

            // [NOTE]
            let (complete, incomplete) = participant.decrypt(known);
            debug_println!("[{}] - Complete: {complete:?}", participant.id);
            debug_println!("[{}] - Incomplete: {incomplete:?}", participant.id);

            let targets = unknown.iter().map(|message| message.target).collect();

            // [NOTE]
            let detected = participant.detect(&targets);
            debug_println!("[{}] - Detected: {detected:?}", participant.id);

            // [NOTE]
            let registered = participant.register(targets);
            debug_println!("[{}] - Registered: {registered:?}", participant.id);

            // [NOTE]
            let queries = participant.forward(
                incomplete
                    .into_iter()
                    .chain(unknown)
                    .chain(registered)
                    .collect(),
            );
            debug_println!("[{}] - Queries: {queries:?}", participant.id);

            debug_println!("--- PARTICIPANT {} END ---", participant.id);

            // [NOTE]
            components.extend(complete);
            sender.send(queries).unwrap();
        }

        println!(
            "Participant {} detected {} components",
            participant.id,
            components.len(),
        );

        results.send((participant.id, components)).unwrap();
    }
}
