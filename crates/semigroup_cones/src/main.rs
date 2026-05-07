//! CLI: write Normaliz input files for the Kunz-cone pair-relations matrices
//! sliced by genus `g`, multiplicity `m`, and Apéry-class parameter `q1`
//! (where `w₁ = q₁·m + 1`), then invoke the bundled Normaliz 3.11.1 binary
//! (`normaliz/normaliz-3.11.1-{Linux,Windows}/normaliz[.exe]`) to produce the
//! corresponding `.out` files, and finally write a single combined HTML
//! summary for g = 2..=gmax.
//!
//! For each `g` and each `m ∈ 2..=(g+1)`, `q1 ∈ 1..=g`, writes
//! `./normaliz/normaliz_g{g}_m{m}_t{q1}.in` (the `_t` infix is a wire-format
//! detail kept stable so cached Normaliz output isn't invalidated) with:
//!
//! - The pair-relations inequalities `(U(m)[i] + U(m)[j] − U(m)[(i+j) mod m]) / m ≥ 0`
//!   (Kunz cone; the ambient variable is `x = C_red[:,0]`, i.e. `x_a = c(a+1, 1)`)
//! - Two affine equalities (Normaliz `inhom_equations` format, row `[a b]` means `a·x + b = 0`):
//!   - `∑xᵢ = m·q1+1`: row 0 of U(m) is all-ones, and `(U·C_red)[0][0] = w₁`, so this pins `w₁`.
//!   - `(1ᵀ U(m))·x = mg+m(m−1)/2`: column sums of U(m) weight x so that `∑wᵢ = selmer`.
//!
//! The lattice points of the resulting polytope correspond bijectively to
//! numerical semigroups with genus `g`, multiplicity `m`, and `w₁ = m·q1+1`.
//!
//! `q1` starts at 1 because `w₁ ≡ 1 (mod m)` and `w₁ ≥ m+1` for any
//! numerical semigroup with `m ≥ 2` (since `1 ∉ S` when the multiplicity is m ≥ 2).
//! The upper bound `q1 ≤ g` follows from `w₁ ≤ ∑wᵢ − ∑_{i=2}^{m−1} i = mg+1`
//! (using `wᵢ ≥ i` for every Apéry element).
//!
//! Usage: `cargo run --bin waldicone [gmax] [seq]`  (gmax defaults to 10)
//! Pass the literal token `seq` as the second argument to force sequential
//! execution (no rayon over the (m, q1) workload or the per-lattice-point
//! post-processing). Useful when Normaliz's own internal threading is
//! already saturating the cores.
//! Computes all genera g = 2..=gmax and writes three output files (HTML in
//! light mode plus a JSON sibling for downstream tooling):
//!  - `./normaliz/semigroup_g_from2to{gmax}_summary.html` — five aggregate
//!    tables (totals, by m, by e, by t, by r) — here `t` is the type (|PF|).
//!  - `./normaliz/semigroup_g_from2to{gmax}_list.html` — one row per
//!    semigroup, ordered by (g, m, q1), with the same shortprops columns
//!    used in the in-app view plus `c_{1,1} … c_{1,gmax}` (zero-padded).
//!  - `./normaliz/semigroup_g_from2to{gmax}_list.json` — the same per-row
//!    data as the list page, one JSON object per line, with full Apéry set
//!    and unpadded `c1` vector for downstream programmatic consumption.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use rayon::prelude::*;
use semigroup_cones::{
    ExecMode, SEQ_FLAG, ensure_normaliz_available, join_row, parse_out_file, run_normaliz,
};
use semigroup_math::math::matrix::u_pair_relations;
use semigroup_math::math::{Semigroup, compute};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Normaliz file generation ──────────────────────────────────────────────────

// ALLOW: pure file-generation pipeline; each block handles one distinct input
// section (ambient space, inequalities, equations) — splitting further would
// obscure the direct correspondence with the Normaliz file format.
#[allow(clippy::too_many_lines)]
fn write_normaliz_files(g: usize, normaliz_bin: &Path, mode: ExecMode) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    // Precompute pair-relations matrices once per m — they don't depend on q1.
    let matrices: Vec<_> = (2..=g + 1).map(u_pair_relations).collect();

    // Skip pairs handled by closed-form shortcuts and pairs ruled out by genus:
    //  • m = 2 has the unique solution ⟨2, 2g+1⟩ (synthesised in HTML).
    //  • m = g+1 has the unique solution ⟨m, m+1, …, 2m−1⟩ (synthesised).
    //  • q1 > g+2−m is empty: such a w₁ forces ≥ q1+(m−2) gaps (proof in todo #40).
    // q1 starts at 1 because w₁ ≡ 1 (mod m) and w₁ ≥ m+1 for m ≥ 2.
    let pairs: Vec<(usize, usize)> = (3..=g)
        .flat_map(|m| (1..=g + 2 - m).map(move |q1| (m, q1)))
        .collect();

    let total = pairs.len();
    let overall = Instant::now();
    let counter = AtomicUsize::new(0);

    let process = |(m, q1): (usize, usize)| -> std::io::Result<()> {
        let n = m - 1;
        let nrows = n * (n + 1) / 2;
        let data = matrices[m - 2].as_slice();

        let mut buf = String::new();
        let _ = writeln!(buf, "amb_space {n}");
        let _ = writeln!(buf, "inequalities {nrows}");
        for r in 0..nrows {
            let _ = writeln!(buf, "{}", join_row((0..n).map(|c| data[r * n + c])));
        }

        // Two affine equalities cut the Kunz cone to a bounded polytope.
        // Normaliz inhom_equations row format: [a₁ … aₙ b] means a·x + b = 0.
        // The ambient variable is x = C_red[:,0], so x_a = c(a+1, 1).
        let selmer = m * g + m * (m - 1) / 2;
        let w1 = m * q1 + 1;

        let _ = writeln!(buf, "inhom_equations 2");

        // Equation 1: ∑xᵢ = w₁  →  [1, 1, …, 1, −w1]
        let mut eq1 = vec![1_i64; n + 1];
        #[allow(clippy::cast_possible_wrap)]
        {
            eq1[n] = -(w1 as i64);
        }
        let _ = writeln!(buf, "{}", join_row(eq1.iter()));

        // Equation 2: (1ᵀ U(m))·x = selmer  →  [col_sums_of_U, −selmer]
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

        // Inhomogeneous inequalities:
        //  (a) Multiplicity-m: κ_a = (w_a − (a+1))/m ≥ 1 for a = 1..n-1.
        //      Without these, κ_a = 0 (w_a = a+1 < m) gives semigroups
        //      with true multiplicity < m — counted in a smaller-m cell.
        //  (b) Upper bound on c_{m-1,1}: from w_{m-1} ≤ selmer − w₁ − Σ_{i=2..m-2}(m+i)
        //      (using the multiplicity lower bounds), get
        //      c_{m-1,1} ≤ (selmer − S_min) / m  where S_min = (m-3)m + (m-2)(m-1)/2 − 1.
        //      LP-derivable from existing constraints, but stating it explicitly
        //      gives Normaliz a tighter bounding box for lattice enumeration.
        //
        //  Note: the symmetric bound c_{1,1} ≤ 2·q1 − 1 was tested and turned out
        //  net-neutral on g=10 (within timing noise) — apparently Normaliz already
        //  derives it from the multiplicity rows + eq1, unlike (b) which
        //  combines eq2 with all multiplicity rows.
        //
        //  Note: the inequality (m−1)·c_{m-1,1} ≥ Σ_{i=1..m-2} c_{i,1} is
        //  algebraically equivalent (via eq1) to c_{m-1,1} ≥ ⌈w₁/m⌉ = q1+1,
        //  which is strictly weaker than what (a) already gives. The last
        //  multiplicity row (a = m−2) forces w_{m-1} ≥ 2m−1, so
        //  c_{m-1,1} = (w_{m-1}+w₁)/m ≥ q1+2. Adding the new inequality
        //  is therefore redundant — same situation as the c_{1,1} ≤ 2·q1−1
        //  bound above.
        #[allow(clippy::cast_possible_wrap)]
        let s_min = if m >= 3 {
            (m - 3) * m + (m - 2) * (m - 1) / 2 - 1
        } else {
            0
        };
        let upper_extra = usize::from(n >= 1);
        let total_ineq = n.saturating_sub(1) + upper_extra;
        if total_ineq > 0 {
            let _ = writeln!(buf, "inhom_inequalities {total_ineq}");
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
            if upper_extra == 1 {
                // Row: −c_{m-1,1} + bound ≥ 0  →  [0 0 … 0 −1 bound]
                #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
                let bound = ((selmer - s_min) / m) as i64;
                let _ = writeln!(
                    buf,
                    "{}",
                    join_row(
                        (0..n)
                            .map(|b| if b == n - 1 { -1 } else { 0 })
                            .chain(std::iter::once(bound))
                    )
                );
            }
        }

        // Filename infix `_t{q1}` is wire-format only; the `_t` letter predates
        // the rename and is kept stable so cached `.out` files stay valid.
        let in_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}.in"));
        let out_path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}.out"));
        let idx = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if out_path.exists() {
            println!("[{idx}/{total}] cached g={g} m={m} q1={q1} (n={n})");
            return Ok(());
        }
        fs::write(&in_path, &buf)?;
        println!("[{idx}/{total}] starting g={g} m={m} q1={q1} (n={n}) ...");
        let started = Instant::now();
        run_normaliz(normaliz_bin, &in_path)?;
        let elapsed = started.elapsed();
        println!(
            "[{idx}/{total}] done g={g} m={m} q1={q1} in {:.2}s (total {:.2}s)",
            elapsed.as_secs_f64(),
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

// ── Generator recovery ────────────────────────────────────────────────────────

/// Recovers the Apéry set `[w₀=0, w₁, …, w_{m−1}]` from `m`, `q1`, and the
/// first column `c1` of `C_red` (the lattice point from Normaliz).
///
/// Uses the recurrence `w_{k+1} = w_k + w₁ − m·c1[k−1]` for k = 1..m−2,
/// with w₁ = m·q1 + 1.
fn apery_from_c1(m: usize, q1: usize, c1: &[i64]) -> Vec<usize> {
    let w1 = m * q1 + 1;
    let mut apery = vec![0usize; m];
    apery[1] = w1;
    for k in 1..m - 1 {
        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        let next = apery[k] as i64 + w1 as i64 - m as i64 * c1[k - 1];
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            apery[k + 1] = next as usize;
        }
    }
    apery
}

/// Returns the [`Semigroup`] whose Apéry set is `apery` (and multiplicity `m`).
///
/// Passes `{m} ∪ {w₁,…,w_{m−1}}` to [`compute`], which finds the minimal
/// generating set by stripping redundant elements and computes all derived
/// invariants (Frobenius, type, pseudo-Frobenius, etc.).
fn semigroup_from_apery(m: usize, apery: &[usize]) -> Semigroup {
    let mut input: Vec<usize> = apery.iter().copied().filter(|&w| w > 0).collect();
    input.push(m);
    compute(&input)
}

/// Closed-form lattice point for the trivially-determined cells `m = 2` and
/// `m = g+1`, returning `(q1, lattice)`.
///
/// • `m = 2`: only ⟨2, 2g+1⟩ has genus g; Apéry = (0, 2g+1), so c₁,₁ = 2g+1
///   and q1 = g.
/// • `m = g+1`: only ⟨m, m+1, …, 2m−1⟩ has genus g; Apéry = (0, m+1, …, 2m−1),
///   so c₁ = (1, 1, …, 1, 3) and q1 = 1.
fn synthetic_lattice(g: usize, m: usize) -> Option<(usize, Lattice)> {
    #[allow(clippy::cast_possible_wrap)]
    if m == 2 {
        let pt = vec![(2 * g + 1) as i64];
        Some((g, (pt, compute(&[2, 2 * g + 1]))))
    } else if m == g + 1 && g >= 2 {
        let mut pt: Vec<i64> = vec![1; m - 2];
        pt.push(3);
        let gens: Vec<usize> = (m..2 * m).collect();
        Some((1, (pt, compute(&gens))))
    } else {
        None
    }
}

/// Renders the shortprops-style data cells for one semigroup: f, e, σ, r, ra,
/// fg, t, Sym, gen (textbox), PF (textbox), Wilf, 1/e.
#[allow(clippy::cast_precision_loss)]
fn props_cells(sg: &Semigroup) -> String {
    let pf_str = sg
        .pf_set
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let gens_str = sg
        .gen_set
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    let glyph = |b: bool| if b { "\u{2705}" } else { "\u{1F6AB}" };
    let sym = glyph(sg.is_symmetric);
    let asym = glyph(sg.is_almost_symmetric);
    let any2 = glyph(sg.any_ri_eq_2());
    let deep = glyph(sg.is_deep());
    format!(
        "<td>{f}</td><td>{e}</td><td>{cg}</td><td>{r}</td><td>{ra}</td>\
         <td>{fg}</td><td>{t}</td><td>{sym}</td><td>{asym}</td><td>{level}</td>\
         <td><input class=\"gens\" type=\"text\" readonly value=\"{gens_str}\"></td>\
         <td><input class=\"pfs\" type=\"text\" readonly value=\"{pf_str}\"></td>\
         <td>{wilf:.4}</td><td>{inv_e:.4}</td>\
         <td>{min_ri}</td><td>{max_ri}</td><td>{any2}</td><td>{deep}</td>",
        f = sg.f,
        e = sg.e,
        cg = sg.count_set,
        r = sg.r,
        ra = sg.ra,
        fg = sg.fg,
        t = sg.t,
        level = sg.level,
        wilf = sg.wilf(),
        inv_e = 1.0 / sg.e as f64,
        min_ri = sg.min_ri(),
        max_ri = sg.max_ri(),
    )
}

// ── HTML generation ───────────────────────────────────────────────────────────

/// One Normaliz lattice point paired with its computed [`Semigroup`].
type Lattice = (Vec<i64>, Semigroup);

/// Per-genus container. Each tuple is `(m, q1, count, lattices)` where `q1`
/// is the Apéry-class parameter (`w₁ = m·q1+1`) and `count` is the lattice-
/// point count reported by Normaliz (matches `lattices.len()` except in
/// closed-form synthetic cases).
type GenusData = Vec<(usize, usize, usize, Vec<Lattice>)>;

/// Writes a row of count cells (one cell per array entry) into `h`. The first
/// cell gets a `sep` border. `is_total` styles non-zero cells with `class=sum`.
fn write_count_cells(h: &mut String, counts: &[usize], is_total: bool) {
    for (idx, &c) in counts.iter().enumerate() {
        let sep = if idx == 0 { " sep" } else { "" };
        if c == 0 {
            let _ = write!(h, "<td class=\"zero{sep}\">\u{b7}</td>");
        } else if is_total {
            let _ = write!(h, "<td class=\"sum{sep}\">{c}</td>");
        } else {
            let _ = write!(h, "<td class=\"{}\">{c}</td>", sep.trim());
        }
    }
}

/// Tally for one genus: scalar invariants and four distributions
/// (by multiplicity m, by embedding dimension e, by type t, by reflected gaps r).
struct GenusTally {
    total: usize,
    zero: usize,
    w1gen: usize,
    sym: usize,
    asym: usize,
    /// Count of semigroups where the largest generator equals f+m equals 2g+1.
    ae_fm_2g_plus_1: usize,
    /// Count of semigroups where the largest generator equals f+m equals 2g.
    ae_fm_2g: usize,
    /// Counts bucketed by where f sits relative to m:
    /// `[f<m, m<f<2m, 2m<f<3m, 3m<f]`. f never equals km for k ≥ 1
    /// (km ∈ S since m is a generator), so the four buckets partition all rows.
    f_vs_m: [usize; 4],
    /// Count of semigroups with `any_ri_eq_2() == true`, i.e. some residue class
    /// has exactly two reflected gaps.
    count_any_ri_eq_2: usize,
    /// Count of deep semigroups: those where all elements m+1 … 2m−1 are gaps.
    deep: usize,
    /// Count of descent semigroups: every Apéry element is either exactly f+m
    /// or strictly less than f (adding f to S is "clean").
    descent: usize,
    by_m: Vec<usize>,
    by_e: Vec<usize>,
    by_t: Vec<usize>,
    by_r: Vec<usize>,
    /// Distribution by `min_ri` (the smallest `r_i` over residue classes 1..m).
    by_min_ri: Vec<usize>,
    /// Distribution by `max_ri` (the largest `r_i` over residue classes 1..m).
    by_max_ri: Vec<usize>,
}

// ALLOW: straight-line accumulation across many independent counters.
#[allow(clippy::too_many_lines)]
fn tally_genus(g: usize, data: &GenusData, cols: usize) -> GenusTally {
    let mut by_m = vec![0usize; cols];
    let mut by_e = vec![0usize; cols];
    let mut by_t = vec![0usize; cols];
    let mut by_r = vec![0usize; cols];
    let mut by_min_ri = vec![0usize; cols];
    let mut by_max_ri = vec![0usize; cols];
    let mut f_vs_m = [0usize; 4];
    let mut count_any_ri_eq_2 = 0usize;
    let mut deep = 0usize;
    let mut descent = 0usize;
    let (mut total, mut zero, mut w1gen, mut sym, mut asym, mut ae_fm_2g_plus_1, mut ae_fm_2g) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for (m, _, _, lats) in data {
        for (pt, sg) in lats {
            total += 1;
            if pt.first() == Some(&0) {
                zero += 1;
            }
            if sg.gen_set.contains(&sg.apery_set[1]) {
                w1gen += 1;
            }
            if sg.is_symmetric {
                sym += 1;
            }
            if sg.is_almost_symmetric {
                asym += 1;
            }
            // Refinement: ae = f + m (i.e. f+m is the largest minimal generator).
            // Two empirical predictions: when ae=f+m=2g+1 then r=m-2,
            // and when ae=f+m=2g then r=m-1. Asserted to surface any counter-examples.
            if sg.max_gen == sg.f + sg.m {
                let fm = sg.f + sg.m;
                if fm == 2 * g + 1 {
                    assert_eq!(
                        sg.r,
                        sg.m - 2,
                        "ae=f+m=2g+1 should imply r=m-2; got g={g} m={} r={} f={} gens={:?}",
                        sg.m,
                        sg.r,
                        sg.f,
                        sg.gen_set,
                    );
                    ae_fm_2g_plus_1 += 1;
                } else if fm == 2 * g {
                    assert_eq!(
                        sg.r,
                        sg.m - 1,
                        "ae=f+m=2g should imply r=m-1; got g={g} m={} r={} f={} gens={:?}",
                        sg.m,
                        sg.r,
                        sg.f,
                        sg.gen_set,
                    );
                    ae_fm_2g += 1;
                }
            }
            let bucket = if sg.f < sg.m {
                0
            } else if sg.f < 2 * sg.m {
                1
            } else if sg.f < 3 * sg.m {
                2
            } else {
                3
            };
            f_vs_m[bucket] += 1;
            if *m < cols {
                by_m[*m] += 1;
            }
            if sg.e < cols {
                by_e[sg.e] += 1;
            }
            if sg.t < cols {
                by_t[sg.t] += 1;
            }
            if sg.r < cols {
                by_r[sg.r] += 1;
            }
            let mn = sg.min_ri();
            if mn < cols {
                by_min_ri[mn] += 1;
            }
            let mx = sg.max_ri();
            if mx < cols {
                by_max_ri[mx] += 1;
            }
            if sg.any_ri_eq_2() {
                count_any_ri_eq_2 += 1;
            }
            if sg.is_deep() {
                deep += 1;
            }
            if sg.is_descent() {
                descent += 1;
            }
        }
    }
    GenusTally {
        total,
        zero,
        w1gen,
        sym,
        asym,
        ae_fm_2g_plus_1,
        ae_fm_2g,
        f_vs_m,
        count_any_ri_eq_2,
        deep,
        descent,
        by_m,
        by_e,
        by_t,
        by_r,
        by_min_ri,
        by_max_ri,
    }
}

/// Renders one g × {axis} distribution table where each row is one genus.
/// `axis` is the column-header label ("m", "e", or "t"); `pick` selects the
/// matching `Vec<usize>` from a [`GenusTally`].
fn build_distribution_table(
    title: &str,
    axis: &str,
    cols: usize,
    all_data: &[(usize, GenusData)],
    pick: impl Fn(&GenusTally) -> Vec<usize>,
) -> String {
    let mut h = String::new();
    let _ = writeln!(h, "<h3>{title}</h3>");
    h.push_str("<table><thead><tr><th class=\"lbl\">g</th>");
    for idx in 0..cols {
        let _ = write!(h, "<th>{axis}={idx}</th>");
    }
    h.push_str("</tr></thead><tbody>");
    let mut grand = vec![0usize; cols];
    for (g, data) in all_data {
        let row = pick(&tally_genus(*g, data, cols));
        for (i, &c) in row.iter().enumerate() {
            grand[i] += c;
        }
        let _ = write!(h, "<tr><td class=\"lbl\">{g}</td>");
        write_count_cells(&mut h, &row, false);
        h.push_str("</tr>\n");
    }
    h.push_str("</tbody><tfoot><tr><th class=\"lbl\">Total</th>");
    write_count_cells(&mut h, &grand, true);
    h.push_str("</tr></tfoot></table>\n");
    h
}

/// Renders the per-genus scalars table, its column legend, and the empirical note.
// ALLOW: this function is mostly inline HTML for one wide table; splitting it
// would scatter column definitions, totals, and the legend across helpers.
#[allow(clippy::too_many_lines)]
fn build_scalars_table(cols: usize, all_data: &[(usize, GenusData)]) -> String {
    let mut h = String::new();
    h.push_str(
        "<table><thead><tr><th class=\"lbl\">g</th>\
         <th>N(g)</th>\
         <th title=\"Count of semigroups with c_{1,1}=0\">N'(g) c<sub>1,1</sub>=0</th>\
         <th title=\"Count of semigroups where w_1 is a minimal generator\">\
         N''(g) w<sub>1</sub>\u{2208}gen</th>\
         <th title=\"Count of symmetric semigroups (t = 1, equivalently g = (f+1)/2)\">\
         N<sub>sym</sub>(g)</th>\
         <th title=\"Count of almost-symmetric semigroups (r = 1)\">\
         N<sub>asym</sub>(g)</th>\
         <th class=\"sep\" title=\"Frobenius f &lt; multiplicity m (the unique \
         ordinary semigroup of genus g)\">f&lt;m</th>\
         <th title=\"m &lt; Frobenius f &lt; 2m\">m&lt;f&lt;2m</th>\
         <th title=\"2m &lt; Frobenius f &lt; 3m\">2m&lt;f&lt;3m</th>\
         <th title=\"3m &lt; Frobenius f\">3m&lt;f</th>\
         <th class=\"sep\" title=\"Largest minimal generator equals f+m equals 2g+1\">\
         ae=f+m=2g+1</th>\
         <th title=\"Largest minimal generator equals f+m equals 2g\">\
         ae=f+m=2g</th>\
         <th class=\"sep\" title=\"Count of semigroups where some residue class i has \
         exactly two reflected gaps (r_i = 2)\">\u{2203}r<sub>i</sub>=2</th>\
         <th class=\"sep\" title=\"Count of deep semigroups: all elements \
         m+1\u{2026}2m\u{2212}1 are gaps (equivalently every Kunz quotient q_i \u{2265} 2)\">\
         N<sub>deep</sub>(g)</th>\
         <th class=\"sep\" title=\"Count of descent semigroups: every Ap\u{e9}ry \
         element is either exactly f+m or strictly less than f\">\
         N<sub>descent</sub>(g)</th>\
         </tr></thead><tbody>",
    );
    let sep_at = |i: usize| {
        if i == 5 || i == 9 || i == 11 || i == 12 || i == 13 {
            "sum sep"
        } else {
            "sum"
        }
    };
    let mut totals = [0usize; 14];
    for (g, data) in all_data {
        let row = tally_genus(*g, data, cols);
        let cells = [
            row.total,
            row.zero,
            row.w1gen,
            row.sym,
            row.asym,
            row.f_vs_m[0],
            row.f_vs_m[1],
            row.f_vs_m[2],
            row.f_vs_m[3],
            row.ae_fm_2g_plus_1,
            row.ae_fm_2g,
            row.count_any_ri_eq_2,
            row.deep,
            row.descent,
        ];
        for (acc, c) in totals.iter_mut().zip(cells.iter()) {
            *acc += *c;
        }
        let _ = write!(h, "<tr><td class=\"lbl\"><a href=\"#g{g}\">{g}</a></td>");
        for (i, c) in cells.iter().enumerate() {
            let _ = write!(h, "<td class=\"{}\">{c}</td>", sep_at(i));
        }
        h.push_str("</tr>\n");
    }
    h.push_str("</tbody><tfoot><tr><th class=\"lbl\">Total</th>");
    for (i, c) in totals.iter().enumerate() {
        let _ = write!(h, "<td class=\"{}\">{c}</td>", sep_at(i));
    }
    h.push_str("</tr></tfoot></table>\n");
    h.push_str(
        "<dl class=\"legend\">\
         <dt>g</dt><dd>genus = number of gaps.</dd>\
         <dt>N(g)</dt><dd>total number of numerical semigroups of genus g.</dd>\
         <dt>N'(g) c<sub>1,1</sub>=0</dt>\
         <dd>those whose reduced Kunz coefficient c<sub>1,1</sub>=0, \
         equivalently w<sub>1</sub>=m+1.</dd>\
         <dt>N''(g) w<sub>1</sub>\u{2208}gen</dt>\
         <dd>those where the smallest non-zero Ap\u{e9}ry element w<sub>1</sub> \
         is a minimal generator.</dd>\
         <dt>N<sub>sym</sub>(g)</dt><dd>symmetric semigroups: type t=1, \
         equivalently g=(f+1)/2.</dd>\
         <dt>N<sub>asym</sub>(g)</dt><dd>almost-symmetric semigroups: \
         f+t=2g, equivalently ra=r and PF(S)\u{2216}{f} equals the set of \
         reflected gaps. Includes the symmetric case (t=1, f+1=2g).</dd>\
         <dt>f&lt;m, m&lt;f&lt;2m, 2m&lt;f&lt;3m, 3m&lt;f</dt>\
         <dd>partition of all semigroups by where the Frobenius number sits \
         relative to the multiplicity. f never equals km for k\u{a0}\u{2265}\u{a0}1 \
         since km\u{a0}\u{2208}\u{a0}S.</dd>\
         <dt>ae=f+m=2g+1, ae=f+m=2g</dt>\
         <dd>counts of semigroups in which f+m is itself the largest minimal \
         generator (a<sub>e</sub>=f+m) and additionally equals 2g+1 or 2g, \
         respectively. Empirically these come paired with r=m\u{2212}2 \
         and r=m\u{2212}1 (asserted at runtime).</dd>\
         <dt>\u{2203}r<sub>i</sub>=2</dt>\
         <dd>semigroups where the count of reflected gaps in some residue class \
         i\u{a0}\u{2208}\u{a0}1..m equals 2 \u{2014} a coarse predicate for \
         the closure step S\u{a0}\u{21a6}\u{a0}S\u{a0}\u{222a}\u{a0}{f}.</dd>\
         <dt>N<sub>deep</sub>(g)</dt>\
         <dd>deep semigroups: all elements m+1\u{2026}2m\u{2212}1 are gaps, \
         equivalently every Kunz quotient q<sub>i</sub>\u{a0}\u{2265}\u{a0}2 \
         (equivalently every Ap\u{e9}ry element w<sub>i</sub>\u{a0}&gt;\u{a0}2m).</dd>\
         <dt>N<sub>descent</sub>(g)</dt>\
         <dd>descent semigroups: every Ap\u{e9}ry element is either exactly f+m \
         or strictly less than f \
         (adding f to S is \u{201c}clean\u{201d}).</dd>\
         </dl>\n\
         <p class=\"note\">Empirical: <code>f&lt;m</code> is always 1 (the unique \
         ordinary semigroup \u{27e8}g+1,\u{2026},2g+1\u{27e9}); \
         <code>m&lt;f&lt;2m</code> matches \
         <a href=\"https://oeis.org/A000071\">OEIS A000071</a> \
         (F(n+2)\u{2212}1) for g\u{a0}=\u{a0}2..10: \
         1,\u{a0}2,\u{a0}4,\u{a0}7,\u{a0}12,\u{a0}20,\u{a0}33,\u{a0}54,\u{a0}88.</p>\n\
         <p class=\"note\">The <code>ae=f+m=2g</code> column is <em>not</em> in \
         bijection with N<sub>asym</sub>(g+1) via the closure map \
         S\u{a0}\u{21a6}\u{a0}S\u{a0}\u{222a}\u{a0}{f}. Reason: for every g\u{a0}\u{2265}\u{a0}2 \
         there is exactly one almost-symmetric S of genus g+1 whose closure is the \
         <em>ordinary</em> semigroup of genus g, and ordinary always satisfies \
         ae=f+m=2g+1 \u{2014} so it lands in the neighbouring \
         <code>ae=f+m=2g+1</code> column instead. (For g+1=2 that S is \
         \u{27e8}3,4,5\u{27e9} itself; for g+1\u{a0}\u{2265}\u{a0}3 it is \
         \u{27e8}g+1,\u{a0}g+2,\u{a0}\u{2026},\u{a0}2g+1,\u{a0}2g+3\u{27e9}.) \
         For g+1\u{a0}=\u{a0}2..10 this accounts for 9 of the 30 \
         almost-symmetric semigroups; the remaining 21 do close into the \
         <code>ae=f+m=2g</code> column at genus g.</p>\n",
    );
    h
}

/// Builds the five "Total semigroups per genus" summary tables: scalars,
/// then distributions by multiplicity m, embedding dimension e, type t,
/// and reflected gaps r.
fn build_grand_summary(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let cols = gmax + 2;
    let mut h = String::new();
    h.push_str("<h2>Total semigroups per genus</h2>\n");

    h.push_str(&build_scalars_table(cols, all_data));

    // (2) By multiplicity, (3) by embedding dim, (4) by type, (5) by reflected gaps
    h.push_str(&build_distribution_table(
        "By multiplicity m",
        "m",
        cols,
        all_data,
        |t| t.by_m.clone(),
    ));
    h.push_str(&build_distribution_table(
        "By embedding dimension e",
        "e",
        cols,
        all_data,
        |t| t.by_e.clone(),
    ));
    h.push_str(&build_distribution_table(
        "By type t",
        "t",
        cols,
        all_data,
        |t| t.by_t.clone(),
    ));
    h.push_str(&build_distribution_table(
        "By reflected gaps r",
        "r",
        cols,
        all_data,
        |t| t.by_r.clone(),
    ));
    h.push_str(&build_distribution_table(
        "By min r\u{1d62} (smallest reflected-gap count over residue classes)",
        "min r\u{1d62}",
        cols,
        all_data,
        |t| t.by_min_ri.clone(),
    ));
    h.push_str(&build_distribution_table(
        "By max r\u{1d62} (largest reflected-gap count over residue classes)",
        "max r\u{1d62}",
        cols,
        all_data,
        |t| t.by_max_ri.clone(),
    ));
    h
}

/// Shared `<head>` + opening `<body>` and `<h1>` for both pages.
fn html_head(title: &str, intro: &str) -> String {
    format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"UTF-8\">\n\
         <title>{title}</title>\n\
         <style>\n\
         *{{box-sizing:border-box}}\n\
         body{{background:#fff;color:#222;font-family:monospace;\
               margin:2em auto;max-width:1400px;padding:0 1.5em}}\n\
         h1,h2,h3{{color:#1a1a8e;margin:.6em 0 .3em}}\n\
         p{{margin:.3em 0 .8em;line-height:1.5}}\n\
         a{{color:#1565c0}}\n\
         table{{border-collapse:collapse;margin:.5em 0;font-size:.87em}}\n\
         th,td{{padding:3px 10px;border:1px solid #ccc;text-align:right}}\n\
         thead th,tfoot td{{background:#f0f4f8;color:#333}}\n\
         th.lbl{{text-align:left;color:#333}}\n\
         td.zero{{color:#ccc}}\n\
         td.pos{{background:#e3f2fd;color:#1565c0;font-weight:bold}}\n\
         td.sum{{color:#1a1a8e;font-weight:bold}}\n\
         input.gens,input.pfs{{font-size:.82em;width:11em;border:1px solid #bbb;\
                     background:#fff;padding:2px 4px;cursor:text}}\n\
         th.sep,td.sep{{border-left:2px solid #888}}\n\
         .scroll{{overflow-x:auto}}\n\
         dl.legend{{display:grid;grid-template-columns:max-content 1fr;gap:.2em .8em;\
                    margin:.4em 0 .8em;font-size:.85em;line-height:1.4}}\n\
         dl.legend dt{{color:#1a1a8e;font-weight:bold;text-align:right}}\n\
         dl.legend dd{{margin:0;color:#333}}\n\
         p.note{{font-size:.85em;color:#555;background:#f8f9fa;\
                 border-left:3px solid #1565c0;padding:.4em .8em;margin:.6em 0}}\n\
         code{{background:#f0f4f8;padding:0 .25em;border-radius:3px}}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         <h1>{title}</h1>\n\
         {intro}",
    )
}

/// Renders the m × q1 count table for one genus: rows m = 2..g+1, columns
/// q1 = 1..g, with row/column totals.
fn build_genus_count_table(g: usize, data: &GenusData) -> String {
    let counts: std::collections::HashMap<(usize, usize), usize> = data
        .iter()
        .map(|(m, q1, _, lats)| ((*m, *q1), lats.len()))
        .collect();

    let mut h = String::new();
    let _ = writeln!(h, "<h2 id=\"g{g}\">Genus g\u{a0}=\u{a0}{g}</h2>");
    h.push_str("<table><thead><tr><th class=\"lbl\">m \\ q\u{2081}</th>");
    for q1 in 1..=g {
        let _ = write!(h, "<th>{q1}</th>");
    }
    h.push_str("<th class=\"sum\">\u{3a3}</th></tr></thead><tbody>");

    let mut col_totals = vec![0usize; g + 1];
    let mut grand = 0usize;
    for m in 2..=g + 1 {
        let _ = write!(h, "<tr><th class=\"lbl\">m\u{a0}=\u{a0}{m}</th>");
        let mut row_sum = 0usize;
        for (q1, ct) in col_totals.iter_mut().enumerate().skip(1).take(g) {
            let c = counts.get(&(m, q1)).copied().unwrap_or(0);
            *ct += c;
            row_sum += c;
            if c == 0 {
                h.push_str("<td class=\"zero\">\u{b7}</td>");
            } else {
                let _ = write!(h, "<td class=\"pos\">{c}</td>");
            }
        }
        grand += row_sum;
        let _ = write!(h, "<td class=\"sum\">{row_sum}</td></tr>");
    }
    h.push_str("</tbody><tfoot><tr><th class=\"lbl\">\u{3a3}</th>");
    for ct in col_totals.iter().skip(1) {
        let _ = write!(h, "<td class=\"sum\">{ct}</td>");
    }
    let _ = writeln!(h, "<td class=\"sum\">{grand}</td></tr></tfoot></table>");
    h
}

/// Renders the body shared by every summary page: the five aggregate tables
/// across all genera, then one m × q1 count table per genus. Title and intro
/// are caller-supplied so the same body can serve unfiltered and filtered
/// (predicate-restricted) views.
fn build_summary_body(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let mut h = String::new();
    h.push_str(&build_grand_summary(gmax, all_data));
    for (g, data) in all_data {
        h.push_str(&build_genus_count_table(*g, data));
    }
    h
}

/// Builds the summary page (every numerical semigroup with genus 2..=gmax),
/// wrapping `build_summary_body` with `<head>`, title/intro, and `</body></html>`.
fn build_summary_html(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let title = format!("Numerical Semigroups \u{2014} genus 2 to {gmax} (summary)");
    let intro = format!(
        "<p>Aggregate counts for all numerical semigroups with\n\
         genus g\u{a0}\u{2208}\u{a0}2..{gmax}. For the full per-semigroup list,\n\
         see <a href=\"semigroup_g_from2to{gmax}_list.html\">\
         semigroup_g_from2to{gmax}_list.html</a>\n\
         (or the JSON export\n\
         <a href=\"semigroup_g_from2to{gmax}_list.json\">\
         semigroup_g_from2to{gmax}_list.json</a>).</p>\n"
    );
    let mut h = html_head(&title, &intro);
    h.push_str(&build_summary_body(gmax, all_data));
    h.push_str("</body></html>\n");
    h
}

/// Builds the list page: one row per semigroup, ordered by (g, m, q1), with
/// shortprops columns followed by `c_{1,1} … c_{1,gmax}` (zero-padded).
fn build_list_html(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let title = format!("Numerical Semigroups \u{2014} genus 2 to {gmax} (list)");
    let intro = format!(
        "<p>One row per numerical semigroup, ordered by (g, m, q\u{2081}). Columns\n\
         c<sub>1,j</sub> are entries of the first column of the reduced Kunz\n\
         matrix C<sub>red</sub>; rows with m\u{2212}1\u{a0}&lt;\u{a0}{gmax} are\n\
         zero-padded to {gmax} columns. For aggregate counts, see\n\
         <a href=\"semigroup_g_from2to{gmax}_summary.html\">\
         semigroup_g_from2to{gmax}_summary.html</a>;\n\
         the same data is also available as JSON in\n\
         <a href=\"semigroup_g_from2to{gmax}_list.json\">\
         semigroup_g_from2to{gmax}_list.json</a>.</p>\n"
    );
    let mut h = html_head(&title, &intro);

    h.push_str(
        "<div class=\"scroll\"><table><thead><tr>\
         <th>g</th>\
         <th title=\"Multiplicity (smallest positive element)\">m</th>\
         <th title=\"Frobenius number\">f</th>\
         <th title=\"Embedding dimension\">e</th>\
         <th title=\"Sporadic elements (count of S below f+1)\">\u{03c3}</th>\
         <th title=\"Reflected gaps\">r</th>\
         <th title=\"Reflected Ap\u{e9}ry\">ra</th>\
         <th title=\"Fundamental gaps\">fg</th>\
         <th title=\"Type (|PF|)\">t</th>\
         <th title=\"Symmetric? (t=1, equivalently f+1=2g)\">Sym</th>\
         <th title=\"Almost-symmetric? (f+t=2g, equivalently ra=r and PF\u{2216}{f}=reflected gaps)\">\
         ASym</th>\
         <th title=\"Level of f: level\u{00b7}m &lt; f &lt; (level+1)\u{00b7}m\">level</th>\
         <th title=\"Minimal generators\">gen</th>\
         <th title=\"Pseudo-Frobenius numbers\">PF</th>\
         <th title=\"Wilf quotient \u{03c3}/(f+1)\">Wilf</th>\
         <th title=\"1/e\">1/e</th>\
         <th title=\"min over i\u{2208}1..m of r_i (reflected gaps in residue class i mod m)\">\
         min r<sub>i</sub></th>\
         <th title=\"max over i\u{2208}1..m of r_i\">max r<sub>i</sub></th>\
         <th title=\"True iff some residue class i has r_i = 2\">\u{2203}r<sub>i</sub>=2</th>\
         <th title=\"Deep: all elements m+1\u{2026}2m\u{2212}1 are gaps \
         (equivalently every Ap\u{e9}ry element w_i&gt;2m, every Kunz quotient q_i\u{2265}2)\">\
         deep</th>",
    );
    for j in 1..=gmax {
        let cls = if j == 1 { " class=\"sep\"" } else { "" };
        let _ = write!(h, "<th{cls}>c<sub>1,{j}</sub></th>");
    }
    h.push_str("</tr></thead><tbody>\n");

    let mut rows: Vec<(usize, usize, usize, &Vec<i64>, &Semigroup)> = Vec::new();
    for (g, data) in all_data {
        for (m, q1, _, lats) in data {
            for (pt, sg) in lats {
                rows.push((*g, *m, *q1, pt, sg));
            }
        }
    }
    rows.sort_unstable_by(|a, b| (a.0, a.1, a.2, a.3).cmp(&(b.0, b.1, b.2, b.3)));

    let mut last_g: Option<usize> = None;
    for (g, m, _q1, c1, sg) in rows {
        // Anchor the first row of each new genus so summary-page links land
        // at the right place.
        let id_attr = if last_g == Some(g) {
            String::new()
        } else {
            last_g = Some(g);
            format!(" id=\"g{g}\"")
        };
        let _ = write!(h, "<tr{id_attr}><td>{g}</td><td>{m}</td>");
        h.push_str(&props_cells(sg));
        for j in 0..gmax {
            let v = c1.get(j).copied().unwrap_or(0);
            let cls = if j == 0 { " class=\"sep\"" } else { "" };
            let _ = write!(h, "<td{cls}>{v}</td>");
        }
        h.push_str("</tr>\n");
    }
    h.push_str("</tbody></table></div>\n</body></html>\n");
    h
}

/// Loads all parsed Normaliz output for genera 2..=gmax.
fn load_all_data(gmax: usize, mode: ExecMode) -> std::io::Result<Vec<(usize, GenusData)>> {
    let dir = Path::new("normaliz");
    let lift = |pt: Vec<i64>, m: usize, q1: usize| -> Lattice {
        let apery = apery_from_c1(m, q1, &pt);
        let sg = semigroup_from_apery(m, &apery);
        (pt, sg)
    };
    let mut all_data: Vec<(usize, GenusData)> = Vec::new();
    for g in 2..=gmax {
        let mut data: GenusData = Vec::new();
        for m in 2..=g + 1 {
            // m=2 and m=g+1 have a unique closed-form solution; skip Normaliz I/O.
            if let Some((q1, lattice)) = synthetic_lattice(g, m) {
                data.push((m, q1, 1, vec![lattice]));
                continue;
            }
            // For m ∈ 3..=g, only q1 ≤ g+2−m can be non-empty (todo #40 pruning).
            for q1 in 1..=g + 2 - m {
                // Filename infix `_t{q1}` is wire-format only — see write_normaliz_files.
                let path = dir.join(format!("normaliz_g{g}_m{m}_t{q1}.out"));
                match parse_out_file(&path) {
                    Ok((count, points)) => {
                        let lattices: Vec<Lattice> = match mode {
                            ExecMode::Parallel => {
                                points.into_par_iter().map(|pt| lift(pt, m, q1)).collect()
                            }
                            ExecMode::Sequential => {
                                points.into_iter().map(|pt| lift(pt, m, q1)).collect()
                            }
                        };
                        data.push((m, q1, count, lattices));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e),
                }
            }
        }
        all_data.push((g, data));
    }
    Ok(all_data)
}

/// Joins a slice of `Display` values with commas and wraps in `[…]`. Used to
/// emit JSON arrays of integers (no escaping required).
fn json_array<T: std::fmt::Display>(xs: &[T]) -> String {
    let mut out = String::from("[");
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{x}");
    }
    out.push(']');
    out
}

/// Builds a JSON file mirroring the per-semigroup detail page: same rows,
/// same (g, m, q1) ordering, same scalar fields plus the gen/PF/Apéry sets and
/// the `c₁` lattice point. Hand-rolled rather than pulling in serde — every
/// value is a number, bool, or array of integers, so escaping isn't a concern.
fn build_list_json(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let mut rows: Vec<(usize, usize, usize, &Vec<i64>, &Semigroup)> = Vec::new();
    for (g, data) in all_data {
        for (m, q1, _, lats) in data {
            for (pt, sg) in lats {
                rows.push((*g, *m, *q1, pt, sg));
            }
        }
    }
    rows.sort_unstable_by(|a, b| (a.0, a.1, a.2, a.3).cmp(&(b.0, b.1, b.2, b.3)));

    let mut out = String::new();
    let _ = writeln!(out, "{{\"gmax\":{gmax},\"semigroups\":[");
    let total = rows.len();
    for (idx, (g, m, q1, c1, sg)) in rows.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let inv_e = 1.0 / sg.e as f64;
        let _ = write!(
            out,
            "{{\"g\":{g},\"m\":{m},\"q1\":{q1},\
             \"f\":{f},\"e\":{e},\"sigma\":{sigma},\
             \"r\":{r},\"ra\":{ra},\"fg\":{fg},\"type\":{type_t},\
             \"sym\":{sym},\"asym\":{asym},\"level\":{level},\
             \"wilf\":{wilf:.6},\"inv_e\":{inv_e:.6},\
             \"max_gen\":{max_gen},\
             \"min_ri\":{min_ri},\"max_ri\":{max_ri},\"any_ri_eq_2\":{any2},\
             \"deep\":{deep},\"descent\":{descent},\
             \"gen\":{gen},\"pf\":{pf},\"apery\":{apery},\
             \"c1\":{c1_arr}}}",
            f = sg.f,
            e = sg.e,
            sigma = sg.count_set,
            r = sg.r,
            ra = sg.ra,
            fg = sg.fg,
            type_t = sg.t,
            sym = sg.is_symmetric,
            asym = sg.is_almost_symmetric,
            level = sg.level,
            wilf = sg.wilf(),
            max_gen = sg.max_gen,
            min_ri = sg.min_ri(),
            max_ri = sg.max_ri(),
            any2 = sg.any_ri_eq_2(),
            deep = sg.is_deep(),
            descent = sg.is_descent(),
            gen = json_array(&sg.gen_set),
            pf = json_array(&sg.pf_set),
            apery = json_array(&sg.apery_set),
            c1_arr = json_array(c1),
        );
        if idx + 1 < total {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("]}\n");
    out
}

fn write_html_files(gmax: usize, all_data: &[(usize, GenusData)]) -> std::io::Result<()> {
    let dir = Path::new("normaliz");

    let summary_path = dir.join(format!("semigroup_g_from2to{gmax}_summary.html"));
    fs::write(&summary_path, build_summary_html(gmax, all_data).as_bytes())?;
    println!("wrote {}", summary_path.display());

    let list_path = dir.join(format!("semigroup_g_from2to{gmax}_list.html"));
    fs::write(&list_path, build_list_html(gmax, all_data).as_bytes())?;
    println!("wrote {}", list_path.display());

    let json_path = dir.join(format!("semigroup_g_from2to{gmax}_list.json"));
    fs::write(&json_path, build_list_json(gmax, all_data).as_bytes())?;
    println!("wrote {}", json_path.display());

    Ok(())
}

/// Diagnostic: for every almost-symmetric (`r = 1`) semigroup S, build
/// S' = S ∪ {f} and report which ones do *not* satisfy `ae(S') = f(S') +
/// m(S') = 2·g(S')`. Prints a header line plus one row per anomaly.
fn print_asym_anomalies(all_data: &[(usize, GenusData)]) {
    println!(
        "\n── almost-symmetric S, S\u{2032} = S \u{222a} {{f}}: \
         is ae(S\u{2032}) = f(S\u{2032}) + m(S\u{2032}) = 2\u{00b7}g(S\u{2032})? ──"
    );
    let fmt = |xs: &[usize]| {
        xs.iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut total = 0usize;
    let mut anomalies = 0usize;
    for (_g, data) in all_data {
        for (_m, _t, _, lats) in data {
            for (_pt, sg) in lats {
                if sg.r != 1 {
                    continue;
                }
                total += 1;
                let mut gens = sg.gen_set.clone();
                gens.push(sg.f);
                let s2 = compute(&gens);
                let g2 = s2.count_gap;
                let fm2 = s2.f + s2.m;
                let ok = s2.max_gen == fm2 && fm2 == 2 * g2;
                if !ok {
                    anomalies += 1;
                    let bucket = if s2.max_gen == fm2 && fm2 == 2 * g2 + 1 {
                        "2g+1"
                    } else if s2.max_gen != fm2 {
                        "ae<f+m"
                    } else {
                        "other"
                    };
                    println!(
                        "  asym ⟨{}⟩ g={} f={} \u{2192} S\u{2032}=⟨{}⟩ \
                         g\u{2032}={g2} f\u{2032}={} m\u{2032}={} ae\u{2032}={} f\u{2032}+m\u{2032}={fm2} [{bucket}]",
                        fmt(&sg.gen_set),
                        sg.count_gap,
                        sg.f,
                        fmt(&s2.gen_set),
                        s2.f,
                        s2.m,
                        s2.max_gen,
                    );
                }
            }
        }
    }
    println!(
        "→ {anomalies}/{total} almost-symmetric semigroups have S\u{2032} \
         that does NOT match ae=f+m=2g\u{2032}."
    );
}

/// Conjecture probe: among S that are almost-symmetric AND have every Apéry
/// element `> 2m`, does `S' = S ∪ {f}` land in the well-behaved regime — namely
/// (a) `f+m` is a minimal generator of S', (b) every Apéry element of S' is
/// `> 2m'`, and (c) every `r_i(S') ∈ {1, 2}`? Tabulate outcomes bucketed by
/// `level(S) ≥ 3` (equivalently `f − m > 2m`), since that may be the missing
/// hypothesis. Prints one summary line per (level≥3, all-checks) outcome.
fn print_asym_apery_shift(all_data: &[(usize, GenusData)]) {
    println!(
        "\n── conjecture probe: S almost-symmetric \u{2227} all w_i>2m, \
         S\u{2032} = S \u{222a} {{f}}. ──"
    );
    let mut buckets: std::collections::BTreeMap<(bool, &str), usize> =
        std::collections::BTreeMap::new();
    let mut total = 0usize;
    for (_g, data) in all_data {
        for (_m, _q1, _, lats) in data {
            for (_pt, sg) in lats {
                if !(sg.is_almost_symmetric && sg.all_apery_gt_2m) {
                    continue;
                }
                total += 1;
                let mut gens = sg.gen_set.clone();
                gens.push(sg.f);
                let s2 = compute(&gens);
                let fm2 = s2.f + s2.m;
                let fm_is_gen = s2.gen_set.contains(&fm2);
                let apery_ok = s2.all_apery_gt_2m;
                // Exclude i = μ (always r_μ = 0); for the others, check r_i ∈ {1, 2}.
                let ri_ok = (1..s2.m)
                    .filter(|&i| i != s2.mu)
                    .all(|i| matches!(s2.r_i(i), 1 | 2));
                let level_ge_3 = sg.level >= 3;
                let label = if fm_is_gen && apery_ok && ri_ok {
                    "all_three_ok"
                } else if !fm_is_gen {
                    "f+m_not_gen"
                } else if !apery_ok {
                    "apery_fail"
                } else {
                    "ri_fail"
                };
                *buckets.entry((level_ge_3, label)).or_insert(0) += 1;
            }
        }
    }
    println!("  total in filter: {total}");
    for ((level_ge_3, label), n) in &buckets {
        let lbl = if *level_ge_3 { "level>=3" } else { "level<3 " };
        println!("  {lbl}  {label:<14}  {n}");
    }
}

/// Histograms of `r_i(S')` values across all S in `asym ∧ all_apery>2m ∧ level≥3`.
/// One bar per residue class i ∈ 1..m'(S'), tabulated globally.
fn print_asym_apery_shift_ri_hist(all_data: &[(usize, GenusData)]) {
    println!("\n── r_i(S\u{2032}) histogram for S asym \u{2227} w_i>2m \u{2227} level\u{2265}3:");
    let mut hist: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    let mut total_classes = 0usize;
    let mut total_s = 0usize;
    for (_g, data) in all_data {
        for (_m, _q1, _, lats) in data {
            for (_pt, sg) in lats {
                if !(sg.is_almost_symmetric && sg.all_apery_gt_2m && sg.level >= 3) {
                    continue;
                }
                let mut gens = sg.gen_set.clone();
                gens.push(sg.f);
                let s2 = compute(&gens);
                total_s += 1;
                for i in 1..s2.m {
                    let v = s2.r_i(i);
                    *hist.entry(v).or_insert(0) += 1;
                    total_classes += 1;
                }
            }
        }
    }
    println!("  S in filter: {total_s}, residue classes counted: {total_classes}");
    for (v, n) in &hist {
        println!("  r_i={v}  {n}");
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
    println!("execution mode: {mode:?}");
    for g in 2..=gmax {
        write_normaliz_files(g, &normaliz_bin, mode).expect("failed to run Normaliz");
    }
    let all_data = load_all_data(gmax, mode).expect("failed to load Normaliz output");
    print_asym_anomalies(&all_data);
    print_asym_apery_shift(&all_data);
    print_asym_apery_shift_ri_hist(&all_data);
    write_html_files(gmax, &all_data).expect("failed to write HTML summary");
    println!("done: summary + list in normaliz/semigroup_g_from2to{gmax}_*.html");
}
