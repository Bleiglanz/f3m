//! CLI: write Normaliz input files for the Kunz-cone pair-relations matrices
//! sliced by genus `g`, multiplicity `m`, and Apéry-class parameter `t`,
//! then invoke the bundled Normaliz 3.11.1 binary
//! (`normaliz/normaliz-3.11.1-{Linux,Windows}/normaliz[.exe]`) to produce the
//! corresponding `.out` files, and finally write a single combined HTML
//! summary for g = 2..=gmax.
//!
//! For each `g` and each `m ∈ 2..=(g+1)`, `t ∈ 1..=g`, writes
//! `./normaliz/normaliz_g{g}_m{m}_t{t}.in` with:
//!
//! - The pair-relations inequalities `(U(m)[i] + U(m)[j] − U(m)[(i+j) mod m]) / m ≥ 0`
//!   (Kunz cone; the ambient variable is `x = C_red[:,0]`, i.e. `x_a = c(a+1, 1)`)
//! - Two affine equalities (Normaliz `inhom_equations` format, row `[a b]` means `a·x + b = 0`):
//!   - `∑xᵢ = mt+1`: row 0 of U(m) is all-ones, and `(U·C_red)[0][0] = w₁`, so this pins `w₁`.
//!   - `(1ᵀ U(m))·x = mg+m(m−1)/2`: column sums of U(m) weight x so that `∑wᵢ = selmer`.
//!
//! The lattice points of the resulting polytope correspond bijectively to
//! numerical semigroups with genus `g`, multiplicity `m`, and `w₁ = mt+1`.
//!
//! `t` starts at 1 because `w₁ ≡ 1 (mod m)` and `w₁ ≥ m+1` for any
//! numerical semigroup with `m ≥ 2` (since `1 ∉ S` when the multiplicity is m ≥ 2).
//! The upper bound `t ≤ g` follows from `w₁ ≤ ∑wᵢ − ∑_{i=2}^{m−1} i = mg+1`
//! (using `wᵢ ≥ i` for every Apéry element).
//!
//! Usage: `cargo run --bin waldicone [gmax]`  (gmax defaults to 10)
//! Computes all genera g = 2..=gmax and writes two HTML files (light mode):
//!  - `./normaliz/semigroup_g_from2to{gmax}_summary.html` — five aggregate
//!    tables (totals, by m, by e, by t, by r).
//!  - `./normaliz/semigroup_g_from2to{gmax}_list.html` — one row per
//!    semigroup, ordered by (g, m, t), with the same shortprops columns
//!    used in the in-app view plus `c_{1,1} … c_{1,gmax}` (zero-padded).

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use f3m::math::matrix::u_pair_relations;
use f3m::math::{Semigroup, compute};
use rayon::prelude::*;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn join_row(iter: impl Iterator<Item = impl ToString>) -> String {
    iter.map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

/// Returns the project-relative path to the bundled Normaliz 3.11.1 binary
/// for the current OS, resolved relative to the current working directory.
/// Linux: `normaliz/normaliz-3.11.1-Linux/normaliz`
/// Windows: `normaliz/normaliz-3.11.1-Windows/normaliz.exe`
fn bundled_normaliz_path() -> PathBuf {
    let (subdir, exe) = if cfg!(target_os = "windows") {
        ("normaliz-3.11.1-Windows", "normaliz.exe")
    } else {
        ("normaliz-3.11.1-Linux", "normaliz")
    };
    Path::new("normaliz").join(subdir).join(exe)
}

/// Spawns `normaliz_bin` with the given input file and waits for completion,
/// capturing stdout/stderr and stdin-isolating the child. Inheriting stdio
/// (the previous behaviour) caused two Windows-only failure modes to be
/// silent: child errors lost in the parallel storm, and a phantom console
/// window flashing for every spawn.
fn run_normaliz(normaliz_bin: &Path, in_path: &Path) -> std::io::Result<()> {
    let output = Command::new(normaliz_bin)
        .arg(in_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(std::io::Error::other(format!(
        "normaliz exited with code {} for {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code().unwrap_or(-1),
        in_path.display(),
        String::from_utf8_lossy(&output.stdout).trim_end(),
        String::from_utf8_lossy(&output.stderr).trim_end(),
    )))
}

// ── Normaliz file generation ──────────────────────────────────────────────────

// ALLOW: pure file-generation pipeline; each block handles one distinct input
// section (ambient space, inequalities, equations) — splitting further would
// obscure the direct correspondence with the Normaliz file format.
#[allow(clippy::too_many_lines)]
fn write_normaliz_files(g: usize, normaliz_bin: &Path) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    // Precompute pair-relations matrices once per m — they don't depend on t.
    let matrices: Vec<_> = (2..=g + 1).map(u_pair_relations).collect();

    // Skip pairs handled by closed-form shortcuts and pairs ruled out by genus:
    //  • m = 2 has the unique solution ⟨2, 2g+1⟩ (synthesised in HTML).
    //  • m = g+1 has the unique solution ⟨m, m+1, …, 2m−1⟩ (synthesised).
    //  • t > g+2−m is empty: such a w₁ forces ≥ t+(m−2) gaps (proof in todo #40).
    // t starts at 1 because w₁ ≡ 1 (mod m) and w₁ ≥ m+1 for m ≥ 2.
    let pairs: Vec<(usize, usize)> = (3..=g)
        .flat_map(|m| (1..=g + 2 - m).map(move |t| (m, t)))
        .collect();

    let total = pairs.len();
    let overall = Instant::now();
    let counter = AtomicUsize::new(0);

    pairs
        .into_par_iter()
        .try_for_each(|(m, t)| -> std::io::Result<()> {
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
            let w1 = m * t + 1;

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
            //  Note: the symmetric bound c_{1,1} ≤ 2t − 1 was tested and turned out
            //  net-neutral on g=10 (within timing noise) — apparently Normaliz already
            //  derives it from the multiplicity rows + eq1, unlike (b) which
            //  combines eq2 with all multiplicity rows.
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

            let in_path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.in"));
            let out_path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.out"));
            let idx = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if out_path.exists() {
                println!("[{idx}/{total}] cached g={g} m={m} t={t} (n={n})");
                return Ok(());
            }
            fs::write(&in_path, &buf)?;
            println!("[{idx}/{total}] starting g={g} m={m} t={t} (n={n}) ...");
            let started = Instant::now();
            run_normaliz(normaliz_bin, &in_path)?;
            let elapsed = started.elapsed();
            println!(
                "[{idx}/{total}] done g={g} m={m} t={t} in {:.2}s (total {:.2}s)",
                elapsed.as_secs_f64(),
                overall.elapsed().as_secs_f64(),
            );
            Ok(())
        })?;
    Ok(())
}

// ── Output parsing ────────────────────────────────────────────────────────────

/// Parses a Normaliz `.out` file, returning the lattice-point count and each
/// point's coordinate vector with the trailing dehomogenization column removed.
///
/// Returns `(0, [])` for infeasible (empty) polytopes.
fn parse_out_file(path: &Path) -> std::io::Result<(usize, Vec<Vec<i64>>)> {
    let content = fs::read_to_string(path)?;
    let first = content.lines().next().unwrap_or_default();
    let count: usize = first
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if count == 0 {
        return Ok((0, Vec::new()));
    }
    let sep_pos = content.find("***").unwrap_or(content.len());
    let after = &content[sep_pos..];
    let marker = format!("{count} lattice points in polytope (module generators):");
    let mut points = Vec::with_capacity(count);
    if let Some(pos) = after.find(&marker) {
        for line in after[pos..].lines().skip(1).take(count) {
            let row: Vec<i64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if !row.is_empty() {
                let n = row.len().saturating_sub(1);
                points.push(row[..n].to_vec());
            }
        }
    }
    Ok((count, points))
}

// ── Generator recovery ────────────────────────────────────────────────────────

/// Recovers the Apéry set `[w₀=0, w₁, …, w_{m−1}]` from `m`, `t`, and the
/// first column `c1` of `C_red` (the lattice point from Normaliz).
///
/// Uses the recurrence `w_{k+1} = w_k + w₁ − m·c1[k−1]` for k = 1..m−2,
/// with w₁ = m·t + 1.
fn apery_from_c1(m: usize, t: usize, c1: &[i64]) -> Vec<usize> {
    let w1 = m * t + 1;
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
/// `m = g+1`, returning `(t, lattice)`.
///
/// • `m = 2`: only ⟨2, 2g+1⟩ has genus g; Apéry = (0, 2g+1), so c₁,₁ = 2g+1
///   and t = g.
/// • `m = g+1`: only ⟨m, m+1, …, 2m−1⟩ has genus g; Apéry = (0, m+1, …, 2m−1),
///   so c₁ = (1, 1, …, 1, 3) and t = 1.
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

    let sym = if sg.is_symmetric() {
        "\u{2705}"
    } else {
        "\u{1F6AB}"
    };

    format!(
        "<td>{f}</td><td>{e}</td><td>{cg}</td><td>{r}</td><td>{ra}</td>\
         <td>{fg}</td><td>{t}</td><td>{sym}</td>\
         <td><input class=\"gens\" type=\"text\" readonly value=\"{gens_str}\"></td>\
         <td><input class=\"pfs\" type=\"text\" readonly value=\"{pf_str}\"></td>\
         <td>{wilf:.4}</td><td>{inv_e:.4}</td>",
        f = sg.f,
        e = sg.e,
        cg = sg.count_set,
        r = sg.r,
        ra = sg.ra,
        fg = sg.fg,
        t = sg.t,
        wilf = sg.wilf(),
        inv_e = 1.0 / sg.e as f64,
    )
}

// ── HTML generation ───────────────────────────────────────────────────────────

/// One Normaliz lattice point paired with its computed [`Semigroup`].
type Lattice = (Vec<i64>, Semigroup);

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
    by_m: Vec<usize>,
    by_e: Vec<usize>,
    by_t: Vec<usize>,
    by_r: Vec<usize>,
}

fn tally_genus(g: usize, data: &GenusData, cols: usize) -> GenusTally {
    let mut by_m = vec![0usize; cols];
    let mut by_e = vec![0usize; cols];
    let mut by_t = vec![0usize; cols];
    let mut by_r = vec![0usize; cols];
    let mut f_vs_m = [0usize; 4];
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
            if sg.is_symmetric() {
                sym += 1;
            }
            if sg.r == 1 {
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
        by_m,
        by_e,
        by_t,
        by_r,
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
         </tr></thead><tbody>",
    );
    let sep_at = |i: usize| if i == 5 || i == 9 { "sum sep" } else { "sum" };
    let mut totals = [0usize; 11];
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
         <dt>N<sub>asym</sub>(g)</dt><dd>almost-symmetric semigroups, \
         identified here by exactly one reflected gap (r=1).</dd>\
         <dt>f&lt;m, m&lt;f&lt;2m, 2m&lt;f&lt;3m, 3m&lt;f</dt>\
         <dd>partition of all semigroups by where the Frobenius number sits \
         relative to the multiplicity. f never equals km for k\u{a0}\u{2265}\u{a0}1 \
         since km\u{a0}\u{2208}\u{a0}S.</dd>\
         <dt>ae=f+m=2g+1, ae=f+m=2g</dt>\
         <dd>counts of semigroups in which f+m is itself the largest minimal \
         generator (a<sub>e</sub>=f+m) and additionally equals 2g+1 or 2g, \
         respectively. Empirically these come paired with r=m\u{2212}2 \
         and r=m\u{2212}1 (asserted at runtime).</dd>\
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

/// Renders the m × t count table for one genus: rows m = 2..g+1, columns
/// t = 1..g, with row/column totals.
fn build_genus_count_table(g: usize, data: &GenusData) -> String {
    let counts: std::collections::HashMap<(usize, usize), usize> = data
        .iter()
        .map(|(m, t, _, lats)| ((*m, *t), lats.len()))
        .collect();

    let mut h = String::new();
    let _ = writeln!(h, "<h2 id=\"g{g}\">Genus g\u{a0}=\u{a0}{g}</h2>");
    h.push_str("<table><thead><tr><th class=\"lbl\">m \\ t</th>");
    for t in 1..=g {
        let _ = write!(h, "<th>{t}</th>");
    }
    h.push_str("<th class=\"sum\">\u{3a3}</th></tr></thead><tbody>");

    let mut col_totals = vec![0usize; g + 1];
    let mut grand = 0usize;
    for m in 2..=g + 1 {
        let _ = write!(h, "<tr><th class=\"lbl\">m\u{a0}=\u{a0}{m}</th>");
        let mut row_sum = 0usize;
        for (t, ct) in col_totals.iter_mut().enumerate().skip(1).take(g) {
            let c = counts.get(&(m, t)).copied().unwrap_or(0);
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

/// Builds the summary page: five aggregate tables across all genera, then one
/// m × t count table per genus.
fn build_summary_html(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let title = format!("Numerical Semigroups \u{2014} genus 2 to {gmax} (summary)");
    let intro = format!(
        "<p>Aggregate counts for all numerical semigroups with\n\
         genus g\u{a0}\u{2208}\u{a0}2..{gmax}. For the full per-semigroup list,\n\
         see <a href=\"semigroup_g_from2to{gmax}_list.html\">\
         semigroup_g_from2to{gmax}_list.html</a>.</p>\n"
    );
    let mut h = html_head(&title, &intro);
    h.push_str(&build_grand_summary(gmax, all_data));
    for (g, data) in all_data {
        h.push_str(&build_genus_count_table(*g, data));
    }
    h.push_str("</body></html>\n");
    h
}

/// Builds the list page: one row per semigroup, ordered by (g, m, t), with
/// shortprops columns followed by `c_{1,1} … c_{1,gmax}` (zero-padded).
fn build_list_html(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let title = format!("Numerical Semigroups \u{2014} genus 2 to {gmax} (list)");
    let intro = format!(
        "<p>One row per numerical semigroup, ordered by (g, m, t). Columns\n\
         c<sub>1,j</sub> are entries of the first column of the reduced Kunz\n\
         matrix C<sub>red</sub>; rows with m\u{2212}1\u{a0}&lt;\u{a0}{gmax} are\n\
         zero-padded to {gmax} columns. For aggregate counts, see\n\
         <a href=\"semigroup_g_from2to{gmax}_summary.html\">\
         semigroup_g_from2to{gmax}_summary.html</a>.</p>\n"
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
         <th title=\"Symmetric?\">Sym</th>\
         <th title=\"Minimal generators\">gen</th>\
         <th title=\"Pseudo-Frobenius numbers\">PF</th>\
         <th title=\"Wilf quotient \u{03c3}/(f+1)\">Wilf</th>\
         <th title=\"1/e\">1/e</th>",
    );
    for j in 1..=gmax {
        let cls = if j == 1 { " class=\"sep\"" } else { "" };
        let _ = write!(h, "<th{cls}>c<sub>1,{j}</sub></th>");
    }
    h.push_str("</tr></thead><tbody>\n");

    let mut rows: Vec<(usize, usize, usize, &Vec<i64>, &Semigroup)> = Vec::new();
    for (g, data) in all_data {
        for (m, t, _, lats) in data {
            for (pt, sg) in lats {
                rows.push((*g, *m, *t, pt, sg));
            }
        }
    }
    rows.sort_unstable_by(|a, b| (a.0, a.1, a.2, a.3).cmp(&(b.0, b.1, b.2, b.3)));

    let mut last_g: Option<usize> = None;
    for (g, m, _t, c1, sg) in rows {
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
fn load_all_data(gmax: usize) -> std::io::Result<Vec<(usize, GenusData)>> {
    let dir = Path::new("normaliz");
    let mut all_data: Vec<(usize, GenusData)> = Vec::new();
    for g in 2..=gmax {
        let mut data: GenusData = Vec::new();
        for m in 2..=g + 1 {
            // m=2 and m=g+1 have a unique closed-form solution; skip Normaliz I/O.
            if let Some((t, lattice)) = synthetic_lattice(g, m) {
                data.push((m, t, 1, vec![lattice]));
                continue;
            }
            // For m ∈ 3..=g, only t ≤ g+2−m can be non-empty (todo #40 pruning).
            for t in 1..=g + 2 - m {
                let path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.out"));
                match parse_out_file(&path) {
                    Ok((count, points)) => {
                        let lattices: Vec<Lattice> = points
                            .into_par_iter()
                            .map(|pt| {
                                let apery = apery_from_c1(m, t, &pt);
                                let sg = semigroup_from_apery(m, &apery);
                                (pt, sg)
                            })
                            .collect();
                        data.push((m, t, count, lattices));
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

fn write_html_files(gmax: usize, all_data: &[(usize, GenusData)]) -> std::io::Result<()> {
    let dir = Path::new("normaliz");

    let summary_path = dir.join(format!("semigroup_g_from2to{gmax}_summary.html"));
    fs::write(&summary_path, build_summary_html(gmax, all_data).as_bytes())?;
    println!("wrote {}", summary_path.display());

    let list_path = dir.join(format!("semigroup_g_from2to{gmax}_list.html"));
    fs::write(&list_path, build_list_html(gmax, all_data).as_bytes())?;
    println!("wrote {}", list_path.display());

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

/// Preflight: confirm the bundled Normaliz binary exists, runs, and is
/// distinct from the calling executable. Returns the **absolute** path so
/// every subsequent spawn is unambiguous — on Windows in particular a bare
/// or relative program name fed to `CreateProcess` searches the directory of
/// the calling executable first, which is `target\release\` and contains the
/// cargo-built `normaliz.exe` (this binary itself). That self-spawn caused
/// the historical fork-bomb where the loop kept restarting the same g/m/t.
fn ensure_normaliz_available() -> PathBuf {
    let rel = bundled_normaliz_path();
    let abs = match rel.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "error: cannot resolve bundled Normaliz at `{}` ({e}). Run \
                 from the project root and make sure the `normaliz/` folder \
                 with the bundled distribution is present.",
                rel.display(),
            );
            std::process::exit(1);
        }
    };

    // Defence against the original Windows fork-bomb: bail loudly if the
    // bundled binary path resolves to this very executable.
    if let Ok(self_path) = std::env::current_exe()
        && let Ok(self_canon) = self_path.canonicalize()
        && self_canon == abs
    {
        eprintln!(
            "error: bundled Normaliz path {} resolves to this binary itself \
             — refusing to self-spawn. Check the `normaliz/` folder layout.",
            abs.display(),
        );
        std::process::exit(1);
    }

    match Command::new(&abs)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => {
            let banner = String::from_utf8_lossy(&out.stdout);
            let version = banner.lines().next().unwrap_or("Normaliz");
            println!("using bundled {version} at {}", abs.display());
            abs
        }
        Ok(out) => {
            eprintln!(
                "error: `{} --version` exited with {}\n--- stderr ---\n{}",
                abs.display(),
                out.status,
                String::from_utf8_lossy(&out.stderr).trim_end(),
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "error: cannot launch bundled Normaliz at {} ({e}). On Linux \
                 ensure the file is executable (`chmod +x {}`); only Linux \
                 and Windows binaries are bundled.",
                abs.display(),
                abs.display(),
            );
            std::process::exit(1);
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let normaliz_bin = ensure_normaliz_available();
    let gmax: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    for g in 2..=gmax {
        write_normaliz_files(g, &normaliz_bin).expect("failed to run Normaliz");
    }
    let all_data = load_all_data(gmax).expect("failed to load Normaliz output");
    print_asym_anomalies(&all_data);
    write_html_files(gmax, &all_data).expect("failed to write HTML summary");
    println!("done: summary + list in normaliz/semigroup_g_from2to{gmax}_*.html");
}
