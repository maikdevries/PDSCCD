use curve25519_dalek::{RistrettoPoint, Scalar, constants::RISTRETTO_BASEPOINT_POINT as G};

struct Crypto {
    private: Scalar,
    public: RistrettoPoint,
}

impl Crypto {
    fn new() -> Self {
        let secret = Scalar::random(&mut rand::rng());

        Self {
            private: secret,
            public: secret * G,
        }
    }

    fn encrypt(&self, message: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            randomness: r * G,
            message: message.point + r * self.public,
        }
    }

    fn decrypt(&self, cipher: Ciphertext) -> Plaintext {
        Plaintext {
            point: cipher.message - self.private * cipher.randomness,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Ciphertext {
    randomness: RistrettoPoint,
    message: RistrettoPoint,
}

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
    fn encode_onto_curve_deterministic() {
        let value = rand::random::<u128>();

        let a = Plaintext::from(value);
        let b = Plaintext::from(value);

        assert_eq!(a, b);
    }

    #[test]
    fn encrypt_probabilistic() {
        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let scheme = Crypto::new();

        let a = scheme.encrypt(&message);
        let b = scheme.encrypt(&message);

        assert_ne!(a, b);
    }

    #[test]
    fn decrypt() {
        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let scheme = Crypto::new();

        let cipher = scheme.encrypt(&message);
        let plain = scheme.decrypt(cipher);

        assert_eq!(plain, message);
    }
}
