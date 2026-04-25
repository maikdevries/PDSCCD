use curve25519_dalek::{RistrettoPoint, Scalar, constants::RISTRETTO_BASEPOINT_POINT as G};

// ---

impl std::fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Ciphertext");
    }
}

impl std::fmt::Debug for Plaintext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Plaintext");
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Plaintext {
    pub point: RistrettoPoint,
}

impl<T> From<T> for Plaintext
where
    T: Into<u128>,
{
    fn from(value: T) -> Self {
        Self {
            point: RistrettoPoint::hash_from_bytes::<sha3::Sha3_512>(&value.into().to_ne_bytes()),
        }
    }
}

impl std::ops::Mul<Scalar> for Plaintext {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self {
            point: self.point * rhs,
        }
    }
}

pub struct Sealed {
    pub cipher: Ciphertext,
    pub nonce: u128,
}

pub struct Unsealed {
    pub nonce: u128,
    pub plain: Plaintext,
}

impl std::ops::Mul<Scalar> for Unsealed {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self {
            nonce: self.nonce,
            plain: self.plain * rhs,
        }
    }
}

pub struct STTP {
    public: RistrettoPoint,
    secret: Scalar,
}

impl STTP {
    pub fn new() -> Self {
        let secret = Scalar::random(&mut rand::rng());

        Self {
            public: secret * G,
            secret: secret,
        }
    }

    pub fn encrypt(&self, message: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: message.point + r * self.public,
            randomness: r * G,
        }
    }

    pub fn rerandomise(&self, cipher: &Ciphertext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            message: cipher.message + r * self.public,
            randomness: cipher.randomness + r * G,
        }
    }

    pub fn unseal(
        &self,
        seals: Vec<Sealed>,
        blinds: Vec<Plaintext>,
    ) -> (Vec<Unsealed>, Vec<Plaintext>) {
        let r = Scalar::random(&mut rand::rng());

        let unsealed = seals
            .into_iter()
            .map(|seal| Unsealed {
                nonce: seal.nonce,
                plain: self.decrypt(&seal.cipher) * r,
            })
            .collect();

        let blinds = blinds.into_iter().map(|blind| blind * r).collect();

        return (unsealed, blinds);
    }

    fn decrypt(&self, cipher: &Ciphertext) -> Plaintext {
        Plaintext {
            point: cipher.message - self.secret * cipher.randomness,
        }
    }
}
