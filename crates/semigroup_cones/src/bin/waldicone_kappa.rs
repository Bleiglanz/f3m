//! `waldicone_kappa`: Kunz-polytope enumeration in κ-coordinates with verification.
//!
//! **STATUS: EXPERIMENTAL — ABANDONED.** This binary explored whether the
//! classical κ-coordinate (Kunz quotient) formulation of the Kunz cone yields
//! simpler constraints that Normaliz solves faster than the c₁ variables used
//! by `waldicone`.  The coefficient improvement is real (see the table below),
//! but g = 20 remains out of reach regardless of the coordinate system, so
//! the experiment is shelved.  Active development concentrates on `waldicone`
//! only.
//!
//! ---
//!
//! Uses κₖ = (wₖ − k)/m (Kunz quotients) as the Normaliz variables instead of
//! the c_{1,k} Kunz-matrix entries used by `waldicone`.  This is the classical
//! Kunz cone from Kunz (1970) / Rosales & García-Sánchez.
//!
//! ## Relationship to `waldicone`
//!
//! The c₁ variables and κ variables are related by the affine invertible transform
//! (over ℤ):
//!
//! ```text
//! c_{1,j} = κ₁ + κⱼ − κ_{j+1}   for j = 1 … m−2  (κ₁ = q₁, κ₀ = 0)
//! c_{1,m-1} = 1 + κ₁ + κ_{m-1}  (sum 1 + (m−1) = m wraps to residue 0)
//! ```
//!
//! ## Pair-condition floor term
//!
//! The Kunz coefficient satisfies `c(a,b) = ⌊(a+b)/m⌋ + κₐ + κᵦ − κ_{(a+b) mod m} ≥ 0`.
//! The floor is 0 for `a+b < m` and 1 for `m < a+b < 2m`.  The condition is therefore:
//! - `a+b < m`: `κₐ + κᵦ ≥ κ_{a+b}`
//! - `a+b > m`: `κₐ + κᵦ + 1 ≥ κ_{a+b−m}` (one unit of slack vs. the naive form)
//!
//! The "+1" is a genuine floor correction, not a rounding artifact — omitting it
//! silently drops all semigroups where equality `κₐ + κᵦ = κ_{a+b−m} − 1` holds.
//!
//! ## Why this is cheaper for Normaliz
//!
//! | Constraint | waldicone (`c₁` variables) | `waldicone_kappa` (κ variables) |
//! |---|---|---|
//! | Kunz cone (pair relations) | ±1 entries (already divided by m) | ±1 entries + floor constant |
//! | Genus / Selmer equation | coefficients up to m(m−1)/2 | Σκ = g (all +1) |
//! | Multiplicity κₖ ≥ 1 | rows with coefficients up to m | κₖ ≥ 1 (coefficient 1) |
//!
//! ## Verification
//!
//! After all Normaliz runs, each κ-point is converted back to c₁ via the formula
//! above and compared with waldicone's base `.out` files (mismatch → exit 1).
//!
//! Filenames: `normaliz/normaliz_kappa_g{g}_m{m}_t{q1}.{in,out}`.
//!
//! Usage: `cargo run --release --bin waldicone_kappa [gmax] [seq]`

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use rayon::prelude::*;
use semigroup_cones::{
    ExecMode, SEQ_FLAG, ensure_normaliz_available, join_row, parse_out_file, run_normaliz,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── κ ↔ c₁ conversion ────────────────────────────────────────────────────────

/// Converts a κ-space lattice point `(κ₂, …, κ_{m−1})` back to the c₁-space
/// vector `(c_{1,1}, …, c_{1,m−1})` used by `waldicone`.
///
/// The bijection over ℤ is:
/// - `c_{1,j} = κ₁ + κⱼ − κ_{j+1}` for j = 1 … m−2
/// - `c_{1,m−1} = 1 + κ₁ + κ_{m−1}` (wrap-around: 1+(m−1) ≡ 0 mod m)
///
/// where κ₁ = q1 (the stratification parameter) and κ₀ = 0 by convention.
fn kappa_to_c1(kappa_free: &[i64], q1: usize) -> Vec<i64> {
    #[allow(clippy::cast_possible_wrap)]
    let q1_i = q1 as i64;
    let n_free = kappa_free.len(); // m − 2
    let n = n_free + 1; // m − 1, number of c₁ entries
    let mut c1 = Vec::with_capacity(n);

    // j = 1: c_{1,1} = κ₁ + κ₁ − κ₂ = 2q₁ − κ₂
    let kappa2 = if n_free > 0 { kappa_free[0] } else { 0 };
    c1.push(2 * q1_i - kappa2);

    // j = 2 … m−2: c_{1,j} = q₁ + κⱼ − κ_{j+1}
    for j in 2..=n_free {
        // κⱼ is kappa_free[j − 2], κ_{j+1} is kappa_free[j − 1]
        c1.push(q1_i + kappa_free[j - 2] - kappa_free[j - 1]);
    }

    // j = m−1: c_{1,m−1} = 1 + q₁ + κ_{m−1} (wrap-around case)
    if n_free > 0 {
        c1.push(1 + q1_i + kappa_free[n_free - 1]);
    }

    c1
}

// ── Normaliz file generation ──────────────────────────────────────────────────

/// Writes and runs the κ-space Normaliz input files for all valid `(m, q₁)`
/// pairs for genus `g`.
///
/// For each `(m, q₁)` the Normaliz polytope has:
/// - `amb_space m−2` (free variables κ₂, …, κ_{m−1})
/// - `inhom_inequalities`: all non-trivial Kunz pair conditions
///   (`κₐ + κᵦ ≥ κ_{(a+b) mod m}`, with κ₁ = q₁ and κ₀ = 0 substituted in)
///   plus multiplicity conditions `κₖ ≥ 1`
/// - `inhom_equations 1`: genus condition `Σ_{k=2}^{m−1} κₖ = g − q₁`
// ALLOW: pure file-generation pipeline — same structure as waldicone's; splitting
// further would obscure the correspondence with the Normaliz file format.
#[allow(clippy::too_many_lines)]
fn write_kappa_files(g: usize, normaliz_bin: &Path, mode: ExecMode) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    let pairs: Vec<(usize, usize)> = (3..=g)
        .flat_map(|m| (1..=g + 2 - m).map(move |q1| (m, q1)))
        .collect();

    let total = pairs.len();
    let overall = Instant::now();
    let counter = AtomicUsize::new(0);

    let process = |(m, q1): (usize, usize)| -> std::io::Result<()> {
        let in_path = dir.join(format!("normaliz_kappa_g{g}_m{m}_t{q1}.in"));
        let out_path = dir.join(format!("normaliz_kappa_g{g}_m{m}_t{q1}.out"));
        let idx = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if out_path.exists() {
            println!("[{idx}/{total}] cached kappa g={g} m={m} q1={q1}");
            return Ok(());
        }

        let n_free = m - 2; // free variables: κ₂, …, κ_{m−1}

        // Collect all inhomogeneous inequality rows.
        // Row format: [coeff_κ₂, …, coeff_κ_{m−1}, constant], condition ≥ 0.
        // Capacity: m*(m-1)/2 pair rows (minus ~m/2 trivial r=0 pairs) + (m-2) multiplicity rows.
        let mut ineq_rows: Vec<Vec<i64>> = Vec::with_capacity(m * (m - 1) / 2 + m);

        // ── Kunz pair conditions ──────────────────────────────────────────────
        // For each pair (a, b) with 1 ≤ a ≤ b ≤ m−1:
        //   constraint κₐ + κᵦ ≥ κᵣ  where r = (a+b) mod m
        //   skip r = 0 (κ₀ = 0, constraint κₐ + κᵦ ≥ 0 is trivially satisfied)
        for a in 1..m {
            for b in a..m {
                let r = (a + b) % m;
                if r == 0 {
                    continue; // trivial
                }

                // row[n_free] is the inhomogeneous constant; row[k] is coeff for κ_{k+2}.
                let mut row = vec![0_i64; n_free + 1];

                // +κₐ: if a = 1 absorb into constant (κ₁ = q₁), else free variable
                if a == 1 {
                    #[allow(clippy::cast_possible_wrap)]
                    {
                        row[n_free] += q1 as i64;
                    }
                } else {
                    row[a - 2] += 1;
                }

                // +κᵦ: similarly
                if b == 1 {
                    #[allow(clippy::cast_possible_wrap)]
                    {
                        row[n_free] += q1 as i64;
                    }
                } else {
                    row[b - 2] += 1;
                }

                // −κᵣ: if r = 1 absorb into constant (−κ₁ = −q₁), else free variable
                if r == 1 {
                    #[allow(clippy::cast_possible_wrap)]
                    {
                        row[n_free] -= q1 as i64;
                    }
                } else {
                    row[r - 2] -= 1;
                }

                // c(a,b) = ⌊(a+b)/m⌋ + κₐ + κᵦ − κᵣ ≥ 0.
                // The floor is 0 when a+b < m, and 1 when a+b > m (since a,b ≤ m−1
                // means a+b ≤ 2m−2 < 2m, so the floor is at most 1).
                if a + b > m {
                    row[n_free] += 1;
                }

                ineq_rows.push(row);
            }
        }

        // ── Multiplicity conditions: κₖ ≥ 1 for k = 2 … m−1 ────────────────
        for k in 2..m {
            let mut row = vec![0_i64; n_free + 1];
            row[k - 2] = 1;
            row[n_free] = -1;
            ineq_rows.push(row);
        }

        let mut buf = String::new();
        let _ = writeln!(buf, "amb_space {n_free}");

        let n_ineq = ineq_rows.len();
        let _ = writeln!(buf, "inhom_inequalities {n_ineq}");
        for row in &ineq_rows {
            let _ = writeln!(buf, "{}", join_row(row.iter()));
        }

        // ── Genus equation: Σ_{k=2}^{m−1} κₖ = g − q₁ ──────────────────────
        #[allow(clippy::cast_possible_wrap)]
        let genus_rhs = (g - q1) as i64;
        let _ = writeln!(buf, "inhom_equations 1");
        let mut eq_row = vec![1_i64; n_free + 1];
        eq_row[n_free] = -genus_rhs;
        let _ = writeln!(buf, "{}", join_row(eq_row.iter()));

        fs::write(&in_path, &buf)?;
        println!("[{idx}/{total}] starting kappa g={g} m={m} q1={q1} (n_free={n_free}) …");
        let started = Instant::now();
        run_normaliz(normaliz_bin, &in_path)?;
        println!(
            "[{idx}/{total}] done    kappa g={g} m={m} q1={q1} \
             in {:.2}s (total {:.2}s)",
            started.elapsed().as_secs_f64(),
            overall.elapsed().as_secs_f64(),
        );
        Ok(())
    };

    match mode {
        ExecMode::Parallel => pairs.into_par_iter().try_for_each(process)?,
        ExecMode::Sequential => pairs.into_iter().try_for_each(process)?,
    }
    Ok(())
}

// ── Verification ──────────────────────────────────────────────────────────────

/// For every `(g, m, q₁)` slice, converts κ-space lattice points back to c₁
/// and asserts they match waldicone's base `.out` file exactly.
///
/// Skips slices where either file is absent (`waldicone` or `waldicone_kappa` has
/// not yet run for those parameters).  Exits with code 1 on any mismatch.
fn verify_kappa_vs_base(gmax: usize, dir: &Path) {
    println!("\n── Verification: κ-space vs waldicone base ───────────────────────────────");
    let mut checked = 0_usize;
    let mut skipped = 0_usize;
    let mut mismatches = 0_usize;

    for g in 2..=gmax {
        for m in 3..=g {
            for q1 in 1..=g + 2 - m {
                let base_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}.out"));
                let base_pts = match parse_out_file(&base_path) {
                    Ok((_, pts)) => pts,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        skipped += 1;
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  ERROR reading base g={g} m={m} q1={q1}: {e}");
                        mismatches += 1;
                        continue;
                    }
                };

                let kappa_path = dir.join(format!("normaliz_kappa_g{g}_m{m}_t{q1}.out"));
                let kappa_pts = match parse_out_file(&kappa_path) {
                    Ok((_, pts)) => pts,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        skipped += 1;
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  ERROR reading kappa g={g} m={m} q1={q1}: {e}");
                        mismatches += 1;
                        continue;
                    }
                };

                // Convert κ-points to c₁-points for comparison.
                let c1_from_kappa: Vec<Vec<i64>> =
                    kappa_pts.iter().map(|kf| kappa_to_c1(kf, q1)).collect();

                let mut base_sorted = base_pts;
                base_sorted.sort_unstable();
                let mut kappa_c1_sorted = c1_from_kappa;
                kappa_c1_sorted.sort_unstable();

                checked += 1;
                if base_sorted != kappa_c1_sorted {
                    eprintln!(
                        "  MISMATCH g={g} m={m} q1={q1}: \
                         base {} pts, kappa {} pts",
                        base_sorted.len(),
                        kappa_c1_sorted.len(),
                    );
                    let kset: BTreeSet<_> = kappa_c1_sorted.iter().collect();
                    for pt in base_sorted.iter().filter(|p| !kset.contains(p)).take(3) {
                        eprintln!("    in base only:  {pt:?}");
                    }
                    let bset: BTreeSet<_> = base_sorted.iter().collect();
                    for pt in kappa_c1_sorted.iter().filter(|p| !bset.contains(p)).take(3) {
                        eprintln!("    in kappa only: {pt:?}");
                    }
                    mismatches += 1;
                }
            }
        }
    }

    println!(
        "checked {checked} slices, skipped {skipped} (missing base or kappa .out), \
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
        write_kappa_files(g, &normaliz_bin, mode).expect("failed to run Normaliz");
    }
    let dir = Path::new("normaliz");
    verify_kappa_vs_base(gmax, dir);
    println!("done.");
}
