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
    pub fn new(participants: Vec<Participant>) -> Self {
        Self {
            participants: participants.into_iter().map(|p| (p.id, p)).collect(),
        }
    }

    pub fn run(self, initiator: PID) -> HashMap<PID, HashMap<NID, Component>> {
        let participant = self
            .participants
            .get(initiator)
            .expect("Particpant must have known ID");

        // [NOTE]
        let jobs = Protocol::seed(participant);
        let mut pending = jobs.len();

        // [NOTE]
        let mut components = HashMap::new();

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
                && let Ok((id, cs)) = r_receiver.recv()
            {
                components.insert(id, cs);
                pending -= 1;
            }
        });

        return components;
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
        results: Sender<(PID, HashMap<NID, Component>)>,
    ) {
        let mut components: HashMap<NID, Component> = HashMap::new();
        let time = std::time::Instant::now();

        // [NOTE]
        for queries in receiver {
            debug_println!();
            debug_println!("--- PARTICIPANT {} START ---", participant.id);

            // [NOTE]
            let (known, unknown) = participant.partition(queries);
            debug_println!("[{}] - Known: {known:?}", participant.id);
            debug_println!("[{}] - Unknown: {unknown:?}", participant.id);

            // [NOTE]
            let (complete, incomplete) = participant.recognise(known);
            debug_println!("[{}] - Complete: {complete:?}", participant.id);
            debug_println!("[{}] - Incomplete: {incomplete:?}", participant.id);

            let targets = unknown.iter().map(|message| message.target).collect();

            // [NOTE]
            let detected = participant.compute(&targets);
            debug_println!("[{}] - Detected: {detected:?}", participant.id);

            // [NOTE]
            let registered = participant.compose(targets);
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

            // [NOTE]
            for (k, v) in complete {
                components.entry(k).or_default().extend(v);
            }

            // [NOTE]
            sender.send(queries).unwrap();
            debug_println!("--- PARTICIPANT {} END ---", participant.id);
        }

        debug_println!(
            "Participant {} detected {} components in {:?}",
            participant.id,
            components.len(),
            time.elapsed()
        );

        results.send((participant.id, components)).unwrap();
    }
}
