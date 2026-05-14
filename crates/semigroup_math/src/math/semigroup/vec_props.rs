//! Inspectors that return a `Vec<usize>` view of S.

use super::Semigroup;

impl Semigroup {
    /// The "blob": sorted list of all reflected gaps (gaps g with f − g also a gap).
    #[must_use]
    pub fn blob(&self) -> Vec<usize> {
        (0..self.f).filter(|&x| self.is_reflected_gap(x)).collect()
    }
}
