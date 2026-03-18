#![warn(clippy::pedantic)]
pub mod eva;
pub mod math;
pub mod jshelpers;

#[cfg(test)]
mod tests {
    use crate::math::{gcd, compute};

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 3), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(9, 0), 9);
    }

    #[test]
    fn test_semigroup_2_3() {
        let s = compute(&[2, 3]);
        assert_eq!(s.e, 2);
        assert_eq!(s.f, 1);
        let s = compute(&[21, 23, 27, 29, 30]);
        assert_eq!(s.blob().len() + s.count_set, s.count_gap);
    }
}
