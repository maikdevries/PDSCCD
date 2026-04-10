use curve25519_dalek::RistrettoPoint;

#[derive(Debug, PartialEq, Eq)]
struct Plaintext {
    point: RistrettoPoint,
}

impl From<u128> for Plaintext {
    fn from(value: u128) -> Self {
        Self {
            point: RistrettoPoint::hash_from_bytes::<sha3::Sha3_512>(&value.to_ne_bytes()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_onto_curve() {
        let value = rand::random::<u128>();

        let a = Plaintext::from(value);
        let b = Plaintext::from(value);

        assert_eq!(a, b);
    }
}
