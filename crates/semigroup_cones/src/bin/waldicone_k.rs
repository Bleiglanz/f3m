//! `waldicone_k`: k*-stratified Kunz-polytope enumeration with verification.
//!
//! **STATUS: EXPERIMENTAL — ABANDONED.** This binary was a performance
//! experiment exploring whether stratifying by k* (the smallest residue class
//! achieving the Frobenius number) could speed up the enumeration.  In
//! practice g = 20 is out of reach regardless of the polytope decomposition,
//! so the experiment is shelved.  Active development concentrates on
//! `waldicone` only.
//!
//! ---
//!
//! Decomposes each `(g, m, q₁)` polytope from waldicone into `m−1`
//! sub-polytopes indexed by `k*`, the **smallest** residue class achieving
//! `max_k(κ_k)` (and therefore the Frobenius number `f = m·κ_{k*} + k* − m`).
//!
//! **Tie-breaking** is encoded upfront as pure inequality constraints:
//!
//! * **Strict** (`k < k*`): `κ_{k*} ≥ κ_k + 1`, i.e. k* strictly beats every
//!   smaller index.  Since all κ values are integers, this is `≥ 1` in Normaliz.
//! * **Non-strict** (`k > k*`): `κ_{k*} ≥ κ_k`.
//!
//! No post-hoc labelling is needed: every semigroup in the lattice of
//! `(g, m, q₁, k*)` has k* as its (smallest) Frobenius-achieving class by
//! construction.  The disjoint union over k* = 1..m−1 covers every semigroup
//! exactly once.
//!
//! **Verification** (run automatically after all Normaliz calls): for every
//! `(g, m, q₁)` the code asserts that the multiset of c₁ vectors in the
//! k*-union equals the multiset produced by waldicone.  A mismatch prints
//! the first differing points and exits with code 1.
//!
//! Filenames: `normaliz/normaliz_g{g}_m{m}_t{q1}_k{kstar}.{in,out}`.
//!
//! Usage: `cargo run --release --bin waldicone_k [gmax] [seq]`

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use rayon::prelude::*;
use semigroup_cones::{
    ExecMode, SEQ_FLAG, ensure_normaliz_available, join_row, parse_out_file, run_normaliz,
};
use semigroup_math::math::matrix::u_pair_relations;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Normaliz file generation ──────────────────────────────────────────────────

// ALLOW: pure file-generation pipeline identical in structure to waldicone's
// write_normaliz_files; splitting further would obscure the correspondence
// with the Normaliz file format.
#[allow(clippy::too_many_lines)]
fn write_kstar_files(g: usize, normaliz_bin: &Path, mode: ExecMode) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    // Precompute pair-relations matrices once per m (same as waldicone).
    let matrices: Vec<_> = (2..=g + 1).map(u_pair_relations).collect();

    // m=2 and m=g+1 have unique closed-form solutions (handled synthetically in
    // waldicone); skip them here too.  q1 pruning: q1 > g+2−m is provably empty.
    // kstar ranges over every residue class 1..m−1.
    let triples: Vec<(usize, usize, usize)> = (3..=g)
        .flat_map(|m| (1..=g + 2 - m).flat_map(move |q1| (1..m).map(move |kstar| (m, q1, kstar))))
        .collect();

    let total = triples.len();
    let overall = Instant::now();
    let counter = AtomicUsize::new(0);

    let process = |(m, q1, kstar): (usize, usize, usize)| -> std::io::Result<()> {
        let n = m - 1;
        let nrows = n * (n + 1) / 2;
        let data = matrices[m - 2].as_slice();

        let mut buf = String::new();
        let _ = writeln!(buf, "amb_space {n}");
        let _ = writeln!(buf, "inequalities {nrows}");
        for r in 0..nrows {
            let _ = writeln!(buf, "{}", join_row((0..n).map(|c| data[r * n + c])));
        }

        // Two affine equalities pin w₁ and the genus (same as waldicone).
        let selmer = m * g + m * (m - 1) / 2;
        let w1 = m * q1 + 1;

        let _ = writeln!(buf, "inhom_equations 2");

        let mut eq1 = vec![1_i64; n + 1];
        #[allow(clippy::cast_possible_wrap)]
        {
            eq1[n] = -(w1 as i64);
        }
        let _ = writeln!(buf, "{}", join_row(eq1.iter()));

        #[allow(clippy::cast_possible_wrap)]
        let (ni, mi) = (n as i64, m as i64);
        let mut eq2 = vec![0_i64; n + 1];
        for (b, coeff) in eq2.iter_mut().enumerate().take(n) {
            #[allow(clippy::cast_possible_wrap)]
            {
                *coeff = ni * (ni + 1) / 2 - mi * (ni - 1 - b as i64);
            }
        }
        #[allow(clippy::cast_possible_wrap)]
        {
            eq2[n] = -(selmer as i64);
        }
        let _ = writeln!(buf, "{}", join_row(eq2.iter()));

        // s_min for the upper bound on c_{m−1,1} (same derivation as waldicone).
        let s_min = if m >= 3 {
            (m - 3) * m + (m - 2) * (m - 1) / 2 - 1
        } else {
            0
        };
        let upper_extra = usize::from(n >= 1);
        let base_ineq = n.saturating_sub(1) + upper_extra;

        // Each k* stratum adds exactly m−2 = n−1 rows:
        //   (kstar−1) strict  +  (m−1−kstar) non-strict  =  m−2.
        let kstar_ineq = n.saturating_sub(1);
        let total_ineq = base_ineq + kstar_ineq;

        if total_ineq > 0 {
            let _ = writeln!(buf, "inhom_inequalities {total_ineq}");

            // ── Base multiplicity lower-bound rows (identical to waldicone) ──
            // κ_a = (w_a − a)/m ≥ 1 for a = 1..n−1: ensures true multiplicity = m.
            if n > 1 {
                for a in 1..n {
                    #[allow(clippy::cast_possible_wrap)]
                    let (ki, min_w) = ((a + 1) as i64, (m + a + 1) as i64);
                    let _ = writeln!(
                        buf,
                        "{}",
                        join_row(
                            (0..n)
                                .map(|b| if b < a { ki - mi } else { ki })
                                .chain(std::iter::once(-min_w))
                        )
                    );
                }
            }

            // ── Base upper bound on c_{m−1,1} (identical to waldicone) ──
            if upper_extra == 1 {
                #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
                let bound = ((selmer - s_min) / m) as i64;
                let _ = writeln!(
                    buf,
                    "{}",
                    join_row(
                        (0..n)
                            .map(|b| if b == n - 1 { -1_i64 } else { 0_i64 })
                            .chain(std::iter::once(bound))
                    )
                );
            }

            // ── k* strict constraints: κ_{kstar} ≥ κ_k + 1 for k < kstar ──
            //
            // κ_{kstar} − κ_k = (kstar−k)·q₁ − Σ_{a=k−1}^{kstar−2} x_a ≥ 1
            //
            // Row: coefficient −1 for variables x_a with a ∈ k−1..kstar−2
            //      (equivalently a+1 ∈ k..kstar−1, i.e. a+1 >= k && a < kstar−1),
            //      constant (kstar−k)·q₁ − 1.
            for k in 1..kstar {
                #[allow(clippy::cast_possible_wrap)]
                let constant = (kstar - k) as i64 * q1 as i64 - 1;
                let _ = writeln!(
                    buf,
                    "{}",
                    join_row(
                        (0..n)
                            .map(|a| -(i64::from(a + 1 >= k && a < kstar - 1)))
                            .chain(std::iter::once(constant))
                    )
                );
            }

            // ── k* non-strict constraints: κ_{kstar} ≥ κ_k for k > kstar ──
            //
            // κ_{kstar} − κ_k = −(k−kstar)·q₁ + Σ_{a=kstar−1}^{k−2} x_a ≥ 0
            //
            // Row: coefficient +1 for variables x_a with a ∈ kstar−1..k−2
            //      (equivalently a+1 ∈ kstar..k−1, i.e. a+1 >= kstar && a < k−1),
            //      constant −(k−kstar)·q₁.
            for k in kstar + 1..m {
                #[allow(clippy::cast_possible_wrap)]
                let constant = -((k - kstar) as i64 * q1 as i64);
                let _ = writeln!(
                    buf,
                    "{}",
                    join_row(
                        (0..n)
                            .map(|a| i64::from(a + 1 >= kstar && a < k - 1))
                            .chain(std::iter::once(constant))
                    )
                );
            }
        }

        let in_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}_k{kstar}.in"));
        let out_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}_k{kstar}.out"));
        let idx = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if out_path.exists() {
            println!("[{idx}/{total}] cached  g={g} m={m} q1={q1} k*={kstar}");
            return Ok(());
        }
        fs::write(&in_path, &buf)?;
        println!("[{idx}/{total}] starting g={g} m={m} q1={q1} k*={kstar} (n={n}) …");
        let started = Instant::now();
        run_normaliz(normaliz_bin, &in_path)?;
        println!(
            "[{idx}/{total}] done    g={g} m={m} q1={q1} k*={kstar} \
             in {:.2}s (total {:.2}s)",
            started.elapsed().as_secs_f64(),
            overall.elapsed().as_secs_f64(),
        );
        Ok(())
    };

    match mode {
        ExecMode::Parallel => triples.into_par_iter().try_for_each(process)?,
        ExecMode::Sequential => triples.into_iter().try_for_each(process)?,
    }
    Ok(())
}

// ── Verification ──────────────────────────────────────────────────────────────

/// For every `(g, m, q₁)` slice, asserts that the union of c₁ vectors across
/// all k* = 1..m−1 exactly equals the c₁ vectors in waldicone's base `.out`
/// file.
///
/// Skips slices where the base file is absent (waldicone hasn't run yet).
/// Exits with code 1 if any mismatch is found, printing the first differing
/// points to help diagnose the error.
fn verify_kstar_vs_base(gmax: usize, dir: &Path) {
    println!("\n── Verification: k*-union vs waldicone base ──────────────────────────────");
    let mut checked = 0_usize;
    let mut skipped = 0_usize;
    let mut mismatches = 0_usize;

    for g in 2..=gmax {
        // Only the non-synthetic range: m=2 and m=g+1 have unique closed-form
        // solutions in waldicone that we don't Normaliz-compute here.
        for m in 3..=g {
            for q1 in 1..=g + 2 - m {
                let base_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}.out"));
                let base_pts = match parse_out_file(&base_path) {
                    Ok((_, pts)) => pts,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        // waldicone hasn't run this slice yet; skip gracefully.
                        skipped += 1;
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  ERROR reading base g={g} m={m} q1={q1}: {e}");
                        mismatches += 1;
                        continue;
                    }
                };

                // Collect lattice points from all k* strata for this (g, m, q1).
                let mut kstar_pts: Vec<Vec<i64>> = Vec::new();
                for kstar in 1..m {
                    let path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}_k{kstar}.out"));
                    match parse_out_file(&path) {
                        Ok((_, pts)) => kstar_pts.extend(pts),
                        // Not yet computed or pruned-empty — treated as 0 points.
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => {
                            eprintln!("  ERROR reading k*={kstar} g={g} m={m} q1={q1}: {e}");
                            mismatches += 1;
                        }
                    }
                }

                let mut base_sorted = base_pts;
                base_sorted.sort_unstable();
                let mut kstar_sorted = kstar_pts;
                kstar_sorted.sort_unstable();

                checked += 1;
                if base_sorted != kstar_sorted {
                    eprintln!(
                        "  MISMATCH g={g} m={m} q1={q1}: \
                         base {} pts, k*-union {} pts",
                        base_sorted.len(),
                        kstar_sorted.len(),
                    );
                    let kset: BTreeSet<_> = kstar_sorted.iter().collect();
                    for pt in base_sorted.iter().filter(|p| !kset.contains(p)).take(3) {
                        eprintln!("    in base only:     {pt:?}");
                    }
                    let bset: BTreeSet<_> = base_sorted.iter().collect();
                    for pt in kstar_sorted.iter().filter(|p| !bset.contains(p)).take(3) {
                        eprintln!("    in k*-union only: {pt:?}");
                    }
                    mismatches += 1;
                }
            }
        }
    }

    println!(
        "checked {checked} slices, skipped {skipped} (no base .out), \
         {} matched, {mismatches} mismatched",
        checked.saturating_sub(mismatches),
    );
    if mismatches > 0 {
        eprintln!("FAILED");
        std::process::exit(1);
    } else if checked > 0 {
        println!("OK");
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let normaliz_bin = ensure_normaliz_available();
    let mut args = std::env::args().skip(1);
    let gmax: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let mode = match args.next().as_deref() {
        Some(s) if s == SEQ_FLAG => ExecMode::Sequential,
        None => ExecMode::Parallel,
        Some(other) => {
            eprintln!("error: unknown second argument {other:?} (expected {SEQ_FLAG:?} or omit)");
            std::process::exit(1);
        }
    };
    println!("execution mode: {mode:?}  gmax={gmax}");
    for g in 2..=gmax {
        write_kstar_files(g, &normaliz_bin, mode).expect("failed to run Normaliz");
    }
    let dir = Path::new("normaliz");
    verify_kstar_vs_base(gmax, dir);
    println!("done.");
}
