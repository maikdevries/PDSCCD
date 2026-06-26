# Privacy-Preserving Detection of Strongly Connected Components in Distributed Graphs

A Rust prototype implementing a decentralised privacy-preserving protocol for detecting strongly connected components (SCCs) across static distributed partial graphs, as proposed in the master's thesis:

> **Privacy-Preserving Detection of Strongly Connected Components in Distributed Graphs**
> TU Delft, 2026.
> https://resolver.tudelft.nl/uuid:5814de06-6a8e-4aa6-9665-35452bcf6156

## Overview

The protocol extends [Tarjan's classic SCC algorithm](https://doi.org/10.1137/0201010) to identify components that span multiple participants' partial graphs, whose union forms a static unweighted simple directed graph. Each participant holds a partial graph and learns only the SCCs relevant to their own subgraph, with no participant's topology revealed to others.

The core observation driving the protocol: any cross-participant SCC must have at least one incoming and one outgoing external edge in each partial graph it spans. Participants therefore compute partial components lying between nodes with incoming external edges and nodes with outgoing external edges, then collaborate by propagating homomorphically encrypted messages along graph edges. Each message carries the set of strongly connected nodes detected so far. A semi-trusted third party serves as the decryption party, but cannot link ciphertexts to their plaintexts due to ciphertext blinding. Homomorphic encryption is built on elliptic curve cryptography via the `curve25519-dalek` crate.

## Build and run

After having installed the [latest stable version of the Rust toolchain](https://www.rust-lang.org/tools/install):

```sh
cargo run -r
```

This will compute the distributed SCCs of a three-participant graph as defined in `src/main.rs`.

## Citation

If you use this work, please cite the thesis:

```bibtex
@mastersthesis{PDSCCD,
	author = {Maik de Vries},
	title  = {Privacy-Preserving Detection of Strongly Connected Components in Distributed Graphs},
	school = {Delft University of Technology},
	year   = {2026},
	url    = {https://resolver.tudelft.nl/uuid:5814de06-6a8e-4aa6-9665-35452bcf6156}
}
```
