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
}

fn main() {
    let parameters = Parameters {
        iterations: 5,
        length: 4,
        nodes: 3,
        participants: 3,
    };

    let participants = generate_chain_graph(&parameters);
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
            let modulus = parameters.participants * parameters.nodes;

            // [NOTE]
            let nodes = ((p * parameters.nodes)..((p + 1) * parameters.nodes)).map(|nid| {
                Node::new(
                    nid as NID,
                    Location::Internal,
                    [((nid + 1) % modulus) as NID],
                )
            });

            // [NOTE]
            let previous = [Node::new(
                (((p * parameters.nodes - 1) + modulus) % modulus) as NID,
                Location::External(ID[(p - 1 + parameters.participants) % parameters.participants]),
                [(p * parameters.nodes) as NID],
            )]
            .into_iter();

            // [NOTE]
            let next = [Node::new(
                (((p + 1) * parameters.nodes) % modulus) as NID,
                Location::External(ID[(p + 1) % parameters.participants]),
                [],
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

fn write_to_file(
    resources: HashMap<PID, Vec<Resources>>,
    parameters: Parameters,
) -> std::io::Result<()> {
    let filename = format!(
        "{}P{}N{}L.json",
        parameters.participants, parameters.nodes, parameters.length
    );

    let file = File::create_new(Path::new("./benchmarks").join(filename))?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &resources)?;
    return Ok(());
}
