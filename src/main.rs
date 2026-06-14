use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, sync::Arc};

use pcd::private::{
    core::{Graph, Location, NID, Node, PID, Participant},
    crypto::Crypto,
    protocol::{Protocol, Resources},
};

struct Parameters {
    iterations: usize,
    length: usize,
    nodes: usize,
    participants: usize,
    topology: Topology,
}

enum Topology {
    Chain,
    Full,
    Hybrid,
}

impl std::fmt::Display for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topology::Chain => write!(f, "C"),
            Topology::Full => write!(f, "F"),
            Topology::Hybrid => write!(f, "H"),
        }
    }
}

fn main() {
    for p in [2, 4, 6, 8] {
        for n in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
            for l in [8, 16, 32, 64, 128, 256, 512, 1024, 2048] {
                run(Parameters {
                    iterations: 5,
                    length: l,
                    nodes: n,
                    participants: p,
                    topology: Topology::Chain,
                });
            }
        }
    }
}

fn run(parameters: Parameters) {
    let participants = match parameters.topology {
        Topology::Chain => generate_chain_graph(&parameters),
        Topology::Full => generate_full_graph(&parameters),
        Topology::Hybrid => generate_hybrid_graph(&parameters),
    };

    let mut resources: HashMap<PID, Vec<Resources>> = HashMap::new();

    for _ in 0..parameters.iterations {
        let (_, rs) = Protocol::new(participants.clone()).run("A");
        for (pid, rs) in rs {
            resources.entry(pid).or_default().push(rs);
        }
    }

    if let Err(_) = write_to_file(resources, parameters) {
        eprintln!();
        eprintln!("WARNING: RESOURCE DETAILS HAVE NOT BEEN WRITTEN TO DISK!");
    };
}

const ID: [&str; 8] = ["A", "B", "C", "D", "E", "F", "G", "H"];

fn generate_chain_graph(parameters: &Parameters) -> Vec<Participant> {
    let crypto = Arc::new(Crypto::new());

    return ID
        .iter()
        .take(parameters.participants)
        .enumerate()
        .map(|(p, pid)| {
            let n = parameters.participants * parameters.nodes;

            // [NOTE]
            let nodes = ((p * parameters.nodes)..((p + 1) * parameters.nodes))
                .map(|nid| Node::new(nid as NID, Location::Internal, vec![((nid + 1) % n) as NID]));

            // [NOTE]
            let previous = [Node::new(
                (((p * parameters.nodes - 1) + n) % n) as NID,
                Location::External(ID[(p - 1 + parameters.participants) % parameters.participants]),
                vec![(p * parameters.nodes) as NID],
            )]
            .into_iter();

            // [NOTE]
            let next = [Node::new(
                (((p + 1) * parameters.nodes) % n) as NID,
                Location::External(ID[(p + 1) % parameters.participants]),
                vec![],
            )]
            .into_iter();

            return Participant::new(
                pid,
                Graph::new(previous.chain(nodes).chain(next).collect()),
                crypto.clone(),
                parameters.length,
            );
        })
        .collect();
}

fn generate_full_graph(parameters: &Parameters) -> Vec<Participant> {
    let crypto = Arc::new(Crypto::new());

    return ID
        .iter()
        .take(parameters.participants)
        .enumerate()
        .map(|(p, pid)| {
            let n = parameters.participants * parameters.nodes;

            // [NOTE]
            let nodes = ((p * parameters.nodes)..((p + 1) * parameters.nodes)).map(|nid| {
                Node::new(
                    nid as NID,
                    Location::Internal,
                    (0..n)
                        .filter_map(|i| i.ne(&nid).then(|| i as NID))
                        .collect(),
                )
            });

            // [NOTE]
            let external = (0..n).filter_map(|i| {
                (!((p * parameters.nodes)..((p + 1) * parameters.nodes)).contains(&i)).then(|| {
                    Node::new(
                        i as NID,
                        Location::External(ID[i / parameters.nodes]),
                        ((p * parameters.nodes)..((p + 1) * parameters.nodes))
                            .map(|i| i as NID)
                            .collect(),
                    )
                })
            });

            return Participant::new(
                pid,
                Graph::new(nodes.chain(external).collect()),
                crypto.clone(),
                parameters.length,
            );
        })
        .collect();
}

fn generate_hybrid_graph(parameters: &Parameters) -> Vec<Participant> {
    let crypto = Arc::new(Crypto::new());

    return ID
        .iter()
        .take(parameters.participants)
        .enumerate()
        .map(|(p, pid)| {
            let n = parameters.participants * parameters.nodes;

            // [NOTE]
            let nodes = ((p * parameters.nodes)..((p + 1) * parameters.nodes) - 1).map(|nid| {
                Node::new(
                    nid as NID,
                    Location::Internal,
                    ((p * parameters.nodes)..((p + 1) * parameters.nodes))
                        .filter_map(|i| i.ne(&nid).then(|| i as NID))
                        .collect(),
                )
            });

            // [NOTE]
            let last = [Node::new(
                ((p + 1) * parameters.nodes - 1) as NID,
                Location::Internal,
                ((p * parameters.nodes)..((p + 1) * parameters.nodes) - 1)
                    .chain(
                        (0..n)
                            .step_by(parameters.nodes)
                            .filter(|i| *i != (p * parameters.nodes)),
                    )
                    .map(|i| i as NID)
                    .collect(),
            )]
            .into_iter();

            // [NOTE]
            let external = (0..n)
                .step_by(parameters.nodes)
                .filter_map(|i| {
                    i.ne(&(p * parameters.nodes)).then(|| {
                        [
                            Node::new(
                                i as NID,
                                Location::External(ID[i / parameters.nodes]),
                                vec![],
                            ),
                            Node::new(
                                ((i + parameters.nodes - 1) % n) as NID,
                                Location::External(ID[i / parameters.nodes]),
                                vec![(p * parameters.nodes) as NID],
                            ),
                        ]
                    })
                })
                .flatten();

            return Participant::new(
                pid,
                Graph::new(nodes.chain(last).chain(external).collect()),
                crypto.clone(),
                parameters.length,
            );
        })
        .collect();
}

fn write_to_file(
    resources: HashMap<PID, Vec<Resources>>,
    parameters: Parameters,
) -> std::io::Result<()> {
    let filename = format!(
        "{}P{}N{}L{}.json",
        parameters.topology, parameters.participants, parameters.nodes, parameters.length
    );

    let file = File::create_new(Path::new("./benchmarks").join(filename))?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &resources)?;
    return Ok(());
}
