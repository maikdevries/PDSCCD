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
            .expect("Participant must have known ID");

        // [NOTE] Compose seed messages to initiate protocol
        let jobs = Protocol::seed(participant);
        let mut pending = jobs.len();

        let mut components = HashMap::new();

        // [NOTE] Spawn dedicated thread per participant
        thread::scope(|scope| {
            let mut channels = HashMap::new();
            let (sender, receiver) = mpsc::channel();

            // [NOTE] Channels to send and receive results from threads to orchestrator
            let (r_sender, r_receiver) = mpsc::channel();

            for (id, participant) in self.participants {
                let (tx, rx) = mpsc::channel();
                channels.insert(id, tx);

                // [NOTE] Clone shared channels for sending messages to orchestrator
                let tx = sender.clone();
                let rtx = r_sender.clone();
                scope.spawn(move || Protocol::work(participant, rx, tx, rtx));
            }

            // [NOTE] Send seed messages to orchestrator, which will dispatch to respective participant
            sender.send(jobs).unwrap();

            // [NOTE] Process and dispatch messsages to respective participants as long as there is work to do
            while pending > 0 {
                let jobs = receiver.recv().unwrap();

                for (id, queries) in jobs {
                    channels[id].send(queries).unwrap();
                    pending += 1;
                }

                pending -= 1;
            }

            pending = channels.len();
            drop(channels);

            // [NOTE] Collect results from each thread (participant)
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
                // [NOTE] Collect all distinct nodes with an incoming external edge
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

        for messages in receiver {
            debug_println!();
            debug_println!("--- PARTICIPANT {} START ---", participant.id);

            // [NOTE] Partition messages into seen and unseen messages
            let (seen, unseen) = participant.partition(messages);
            debug_println!("[{}] - Seen: {seen:?}", participant.id);
            debug_println!("[{}] - Unseen: {unseen:?}", participant.id);

            // [NOTE] Attempt recognition of seen messages through interaction with the STTP
            let (complete, incomplete) = participant.recognise(seen);
            debug_println!("[{}] - Complete: {complete:?}", participant.id);
            debug_println!("[{}] - Incomplete: {incomplete:?}", participant.id);

            let targets = unseen.iter().map(|message| message.target).collect();

            // [NOTE] Compute paths for unseen target nodes
            let computed = participant.compute(&targets);
            debug_println!("[{}] - Computed: {computed:?}", participant.id);

            // [NOTE] Compose messages for unseen target nodes
            let composed = participant.compose(targets);
            debug_println!("[{}] - Composed: {composed:?}", participant.id);

            // [NOTE] Prepare to forward union of incomplete + unseen + composed messages to respective participants
            let messages = participant.forward(
                incomplete
                    .into_iter()
                    .chain(unseen)
                    .chain(composed)
                    .collect(),
            );
            debug_println!("[{}] - Messages: {messages:?}", participant.id);

            // [NOTE] Store dedected distributed components per root node
            for (k, v) in complete {
                components.entry(k).or_default().extend(v);
            }

            // [NOTE] Send messages back to the orchestrator to dispatch to other participants
            sender.send(messages).unwrap();
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
