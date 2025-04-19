pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub const fn new() -> Self {
        Self {
            state: 0x1234_ABCD_EF01_5678,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_in_range(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next_u64() % (max - min + 1))
    }
}
