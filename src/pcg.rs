use rand::{Rng, SeedableRng, Rand};

/// A [PCG](http://www.pcg-random.org)-based random number generator.
///
/// The PCG algorithm is not suitable for cryptographic purposes but provides an excellent
/// combination of speed and unpredictability. It is only slightly slower than `rand::XorShiftRng`
/// but provides much higher-quality output. In addition, it also provides for the use of multiple
/// distinct _streams_ of outputs given a common seed.
///
/// This particular implementation uses a 128-bit state value, has a period of 2^64, and uses the
/// `XSH-RR` output function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcgRng {
    state: u64,
    inc: u64,
}

impl PcgRng {
    /// Returns a new `PcgRng` instance which is not seeded.
    ///
    /// The initial values of this RNG are constants, so all generators created by this function
    /// will yield the same stream of random numbers. It is highly recommended that this is created
    /// through `SeedableRng` instead of this function.
    pub fn new_unseeded() -> PcgRng {
        PcgRng {
            state: 0x853c49e6748fea9b,
            inc: 0xda3e39cb94b95bdb,
        }
    }

    /// Sets the stream ID of the `PcgRng`.
    pub fn set_stream(&mut self, id: u64) {
        self.inc = id;
    }

    /// Returns a new `PcgRng` instance with the same state as `self`, but with the given stream
    /// ID.
    pub fn with_stream(&self, id: u64) -> PcgRng {
        PcgRng {
            state: self.state,
            inc: id,
        }
    }
}

impl Rng for PcgRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6364136223846793005)
                        .wrapping_add(self.inc);
        let xor = (((old >> 18) ^ old) >> 27) as u32;
        let rot = old >> 59 as u32;
        let out = (xor >> rot) | (xor << (((0 as u64).wrapping_sub(rot)) & 31));
        out
    }
}

impl SeedableRng<[u64; 2]> for PcgRng {
    /// Reseed a `PcgRng`.
    fn reseed(&mut self, seed: [u64; 2]) {
        self.state = 0;
        self.inc = (seed[1] << 1) | 1;
        self.next_u32();
        self.state = self.state.wrapping_add(seed[0]);
        self.next_u32();
    }

    /// Create a new `PcgRng`.
    fn from_seed(seed: [u64; 2]) -> PcgRng {
        let mut rng = PcgRng::new_unseeded();
        rng.reseed(seed);
        rng
    }
}

impl Rand for PcgRng {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        PcgRng { state: rng.next_u64(), inc: rng.next_u64() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rand::{Rng, SeedableRng};

    #[test]
    fn output() {
        let mut rng = PcgRng::from_seed([42, 54]);

        let v: Vec<u32> = rng.gen_iter().take(6).collect();

        // test vectors from pcg32-global-demo
        assert_eq!(v,
                   vec![0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e]);
    }

    #[test]
    fn overflow() {
        let mut rng = PcgRng::from_seed([!0, 54]);
        rng.next_u32();
    }
}
