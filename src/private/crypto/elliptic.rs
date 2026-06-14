use curve25519_dalek::{RistrettoPoint, Scalar, constants::RISTRETTO_BASEPOINT_POINT as G};
use std::collections::HashMap;

use crate::private::core::NID;

// ---

impl std::fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Ciphertext");
    }
}

// ---

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Ciphertext {
    message: RistrettoPoint,
    randomness: RistrettoPoint,
}

impl std::ops::Mul<Scalar> for Ciphertext {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self {
            message: self.message * rhs,
            randomness: self.randomness * rhs,
        }
    }
}

pub type Plaintext = RistrettoPoint;

pub struct Sealed {
    pub nonce: u128,
    pub token: Ciphertext,
}

pub struct Unsealed {
    pub nonce: u128,
    pub token: Plaintext,
}

pub struct Elliptic {
    lookup: HashMap<[u8; 32], NID>,
    public: RistrettoPoint,
    secret: Scalar,
}

impl Elliptic {
    // [NOTE]
    const B: NID = NID::MAX.isqrt() + 1;

    pub fn new() -> Self {
        let secret = Scalar::random(&mut rand::rng());

        Self {
            lookup: Self::generate_lookup(),
            public: Self::encode(secret),
            secret: secret,
        }
    }

    fn generate_lookup() -> HashMap<[u8; 32], NID> {
        let mut lookup = HashMap::with_capacity(Self::B as usize);
        let mut baby = RistrettoPoint::default();

        // [NOTE]
        for i in 0..Self::B {
            lookup.insert(baby.compress().to_bytes(), i);
            baby += G;
        }

        return lookup;
    }

    pub fn encode<T>(message: T) -> Plaintext
    where
        T: Into<Scalar>,
    {
        return RistrettoPoint::mul_base(&message.into());
    }

    pub fn encrypt(&self, plaintext: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: plaintext + r * self.public,
            randomness: Self::encode(r),
        }
    }

    pub fn decrypt(&self, cipher: &Ciphertext) -> Plaintext {
        return cipher.message - self.secret * cipher.randomness;
    }

    pub fn rerandomise(&self, cipher: &Ciphertext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: cipher.message + r * self.public,
            randomness: cipher.randomness + Self::encode(r),
        }
    }

    pub fn unseal(&self, seals: Vec<Sealed>, blind: Plaintext) -> (Vec<Unsealed>, Plaintext) {
        let r = Scalar::random(&mut rand::rng());

        // [NOTE]
        let unsealed = seals
            .into_iter()
            .map(|seal| Unsealed {
                nonce: seal.nonce,
                token: self.decrypt(&seal.token) * r,
            })
            .collect();

        // [NOTE]
        let blind = blind * r;

        return (unsealed, blind);
    }

    pub fn recover(&self, point: &Plaintext) -> Option<NID> {
        let mut giant = *point;

        // [NOTE]
        for i in 0..=Self::B {
            if let Some(j) = self.lookup.get(&giant.compress().to_bytes()) {
                return Some(i * Self::B + j);
            }

            giant -= Self::encode(Self::B);
        }

        return None;
    }
}
