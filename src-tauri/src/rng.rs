pub fn next_u64(state: &mut u64) -> u64 {
    let mut value = if *state == 0 {
        0x9E3779B97F4A7C15
    } else {
        *state
    };
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    value.wrapping_mul(0x2545F4914F6CDD1D)
}

pub fn next_f64(state: &mut u64) -> f64 {
    next_u64(state) as f64 / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::{next_f64, next_u64};

    #[test]
    fn seeded_stream_is_reproducible() {
        let mut left = 42;
        let mut right = 42;
        assert_eq!(next_u64(&mut left), next_u64(&mut right));
        assert_eq!(next_f64(&mut left), next_f64(&mut right));
    }
}
