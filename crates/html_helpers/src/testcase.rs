//! Emit a single semigroup as one CSV row in the
//! `tests/data/check_fixtures.csv` format.
//!
//! Columns: `gens;e;f;m;g;c;sym;apery;pf;t` — inner arrays space-separated,
//! `pf` sorted ascending, `apery` in residue-class (index) order. The
//! parser side lives in `semigroup_math/tests/semigroup_tests.rs`
//! (`test_csv_fixtures`); the unit test below guards against format drift
//! by round-tripping the first fixture row.

use semigroup_math::math::Semigroup;

/// Render `sg` as one CSV row for `tests/data/check_fixtures.csv`.
#[must_use]
pub fn testcase_csv(sg: &Semigroup) -> String {
    let space = |v: &[usize]| v.iter().map(usize::to_string).collect::<Vec<_>>().join(" ");
    let mut pf = sg.pf_set.clone();
    pf.sort_unstable();
    format!(
        "{gens};{e};{f};{m};{g};{c};{sym};{apery};{pf};{t}",
        gens = space(&sg.gen_set),
        e = sg.e,
        f = sg.f,
        m = sg.m,
        g = sg.g,
        c = sg.sigma,
        sym = sg.is_symmetric,
        apery = space(&sg.apery_set),
        pf = space(&pf),
        t = sg.t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use semigroup_math::math::compute;

    /// The first fixture row in `check_fixtures.csv` is `<6, 9, 20>`. Building
    /// it via `compute` and round-tripping through `testcase_csv` must
    /// reproduce that row byte-for-byte; otherwise the parser side will silently
    /// stop accepting our output.
    #[test]
    fn round_trip_matches_first_fixture_row() {
        let sg = compute(&[6, 9, 20]);
        let expected = "6 9 20;3;43;6;22;22;true;0 49 20 9 40 29;43;1";
        assert_eq!(testcase_csv(&sg), expected);
    }
}
