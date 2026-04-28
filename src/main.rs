use pcd::private::{
    core::{Graph, Location, Node, Participant},
    crypto::Crypto,
    protocol::Protocol,
};

fn main() {
    let sttp = Crypto::new();

    let participants = [
        Participant::new(
            "A",
            Graph::new([
                Node::new(0, Location::External("C"), [1]),
                Node::new(1, Location::Internal, [2]),
                Node::new(2, Location::Internal, [3]),
                Node::new(3, Location::Internal, [4]),
                Node::new(4, Location::External("B"), []),
                Node::new(6, Location::External("C"), [7]),
                Node::new(7, Location::Internal, [2]),
            ]),
            &sttp,
            6,
        ),
        Participant::new(
            "B",
            Graph::new([
                Node::new(3, Location::External("A"), [4]),
                Node::new(4, Location::Internal, [5]),
                Node::new(5, Location::Internal, [6]),
                Node::new(6, Location::External("C"), [4]),
            ]),
            &sttp,
            6,
        ),
        Participant::new(
            "C",
            Graph::new([
                Node::new(0, Location::Internal, [1]),
                Node::new(1, Location::External("A"), []),
                Node::new(4, Location::External("B"), []),
                Node::new(5, Location::External("B"), [6]),
                Node::new(6, Location::Internal, [4, 7]),
                Node::new(7, Location::External("A"), []),
            ]),
            &sttp,
            6,
        ),
    ];

    let start = std::time::Instant::now();
    let components = Protocol::new(participants).run("A");

    println!();
    println!("Components: {components:?}");
    println!("Duration: {:?}", std::time::Instant::now() - start);
}
