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
    randomness: RistrettoPoint,
    message: RistrettoPoint,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Plaintext {
    point: RistrettoPoint,
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

pub struct Partial {
    index: usize,
    value: RistrettoPoint,
}

struct Share {
    index: usize,
    value: Scalar,
}

pub struct Threshold {
    private: Share,
    public: RistrettoPoint,
}

impl Threshold {
    pub fn setup<const T: usize, const N: usize>() -> [Self; N] {
        // [NOTE]
        // [BUG] Assert N >= T >= 2
        let coefficients = [0; T].map(|_| Scalar::random(&mut rand::rng()));
        let public = coefficients[0] * G;

        // [NOTE]
        return std::array::from_fn(|i| Self {
            public: public,
            private: Share {
                index: i + 1,
                value: Threshold::horner(&coefficients, Scalar::from(i as u128 + 1)),
            },
        });
    }

    fn horner(coefficients: &[Scalar], x: Scalar) -> Scalar {
        // [NOTE]
        return coefficients
            .iter()
            .rev()
            .fold(Scalar::ZERO, |sum, coefficient| sum * x + coefficient);
    }

    pub fn encrypt(&self, message: &Plaintext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            randomness: r * G,
            message: message.point + r * self.public,
        }
    }

    pub fn rerandomise(&self, cipher: &Ciphertext) -> Ciphertext {
        let r = Scalar::random(&mut rand::rng());

        Ciphertext {
            randomness: cipher.randomness + r * G,
            message: cipher.message + r * self.public,
        }
    }

    pub fn decrypt(&self, cipher: &Ciphertext) -> Partial {
        Partial {
            index: self.private.index,
            value: self.private.value * cipher.randomness,
        }
    }

    pub fn combine(partials: Vec<Partial>, cipher: &Ciphertext) -> Plaintext {
        // [NOTE]
        // [BUG] Assert number of partials == T, else slice
        let coefficients = Threshold::lagrange(
            partials
                .iter()
                .map(|partial| Scalar::from(partial.index as u128))
                .collect(),
        );

        // [NOTE]
        let secret = partials.into_iter().zip(coefficients).fold(
            RistrettoPoint::default(),
            |point, (partial, coefficient)| {
                return point + coefficient * partial.value;
            },
        );

        Plaintext {
            point: cipher.message - secret,
        }
    }

    fn lagrange(indices: Vec<Scalar>) -> Vec<Scalar> {
        // [PERF] Use Montgomery's batch inversion trick to reduce to single inversion
        return indices
            .iter()
            .map(|x| {
                let (numerator, denominator) = indices.iter().filter(|y| *y != x).fold(
                    (Scalar::ONE, Scalar::ONE),
                    |(a, b), y| {
                        return (a * -y, b * (x - y));
                    },
                );

                // [TODO] Assert denominator is non-zero
                return numerator * denominator.invert();
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::IndexedRandom;

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
        let participants = Threshold::setup::<3, 8>();

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        assert_ne!(a, b);
    }

    #[test]
    fn combine() {
        let participants = Threshold::setup::<3, 8>();

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let cipher = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let partials: Vec<Partial> = participants
            .sample(&mut rand::rng(), 3)
            .map(|participant| participant.decrypt(&cipher))
            .collect();

        let plain = Threshold::combine(partials, &cipher);

        assert_eq!(plain, message);
    }

    #[test]
    fn combine_insufficient() {
        let participants = Threshold::setup::<3, 8>();

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let cipher = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let partials: Vec<Partial> = participants
            .sample(&mut rand::rng(), rand::random_range(1..3))
            .map(|participant| participant.decrypt(&cipher))
            .collect();

        let plain = Threshold::combine(partials, &cipher);

        assert_ne!(plain, message);
    }

    #[test]
    fn rerandomise_indistinguishable() {
        let participants = Threshold::setup::<3, 8>();

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .rerandomise(&a);

        assert_ne!(a, b);
    }

    #[test]
    fn rerandomise_preserving() {
        let participants = Threshold::setup::<3, 8>();

        let value = rand::random::<u128>();
        let message = Plaintext::from(value);

        let a = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .encrypt(&message);

        let b = participants
            .choose(&mut rand::rng())
            .expect("Participants should not be empty")
            .rerandomise(&a);

        let partials: Vec<Partial> = participants
            .sample(&mut rand::rng(), 3)
            .map(|participant| participant.decrypt(&b))
            .collect();

        let plain = Threshold::combine(partials, &b);

        assert_eq!(plain, message);
    }
}
