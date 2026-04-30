//! CLI: write Normaliz input files for the Kunz-cone pair-relations matrices
//! sliced by genus `g`, multiplicity `m`, and Apéry-class parameter `t`,
//! then invoke the `normaliz` binary to produce the corresponding `.out` files,
//! and finally write a single combined HTML summary for g = 2..=gmax.
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
//! Usage: `cargo run --bin normaliz [gmax]`  (gmax defaults to 10)
//! Computes all genera g = 2..=gmax and writes one HTML file
//! `./normaliz/semigroup_g_from2to{gmax}.html` (light mode).

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use f3m::math::matrix::u_pair_relations;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn join_row(iter: impl Iterator<Item = impl ToString>) -> String {
    iter.map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

// ── Normaliz file generation ──────────────────────────────────────────────────

// ALLOW: pure file-generation pipeline; each block handles one distinct input
// section (ambient space, inequalities, equations) — splitting further would
// obscure the direct correspondence with the Normaliz file format.
#[allow(clippy::too_many_lines)]
fn write_normaliz_files(g: usize) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    // Precompute pair-relations matrices once per m — they don't depend on t.
    let matrices: Vec<_> = (2..=g + 1).map(u_pair_relations).collect();

    // t=0 (w₁=1) is omitted: 1 ∈ S forces m=1, which is out of scope.
    let pairs: Vec<(usize, usize)> = (2..=g + 1)
        .flat_map(|m| (1..=g).map(move |t| (m, t)))
        .collect();

    let total = pairs.len();
    let overall = Instant::now();
    // TEMPORARY: sequential loop with progress prints to identify slow pairs.
    // Restore `pairs.into_par_iter().try_for_each(...)` once timing is understood.
    for (idx, (m, t)) in pairs.into_iter().enumerate() {
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
        // U(m) row 0 is all-ones and (U·C_red)[0][0] = w₁, so (1,…,1)·x = w₁.
        let mut eq1 = vec![1_i64; n + 1];
        // ALLOW: w1 = mt+1 ≤ mg+1 for any reasonable genus — well within i64::MAX
        #[allow(clippy::cast_possible_wrap)]
        {
            eq1[n] = -(w1 as i64);
        }
        let _ = writeln!(buf, "{}", join_row(eq1.iter()));

        // Equation 2: (1ᵀ U(m))·x = selmer  →  [col_sums_of_U, −selmer]
        // Column b of U(m) sums to n(n+1)/2 − m(n−1−b); weighting x by these
        // sums recovers ∑wᵢ = selmer via ∑_a (U·C_red)[a][0] = ∑wᵢ.
        #[allow(clippy::cast_possible_wrap)]
        let (ni, mi) = (n as i64, m as i64);
        let mut eq2 = vec![0_i64; n + 1];
        for (b, coeff) in eq2.iter_mut().enumerate().take(n) {
            #[allow(clippy::cast_possible_wrap)]
            {
                *coeff = ni * (ni + 1) / 2 - mi * (ni - 1 - b as i64);
            }
        }
        // ALLOW: selmer = mg+m(m-1)/2 is genus-scale, far below i64::MAX
        #[allow(clippy::cast_possible_wrap)]
        {
            eq2[n] = -(selmer as i64);
        }
        let _ = writeln!(buf, "{}", join_row(eq2.iter()));

        // Multiplicity-m constraint: κ_a = (w_a − (a+1))/m ≥ 1 for a = 1..n-1,
        // i.e. w_a ≥ m + a + 1, encoded as U(m)[a]·x ≥ m + a + 1.
        // (a=0 holds automatically: w₁ = mt+1 ≥ m+1 for t ≥ 1.)
        // Without these, the cone admits κ_a = 0 (w_a = a+1 < m), which gives
        // semigroups whose true multiplicity is < m — counted in a smaller-m cell.
        if n > 1 {
            let _ = writeln!(buf, "inhom_inequalities {}", n - 1);
            for a in 1..n {
                // U(m)[a][b] = a+1 if b ≥ a, else a+1−m.
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

        let in_path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.in"));
        fs::write(&in_path, &buf)?;

        println!(
            "[{}/{total}] starting g={g} m={m} t={t} (n={n}, nrows={nrows}) ...",
            idx + 1,
        );
        let started = Instant::now();
        let status = Command::new("normaliz").arg(&in_path).status()?;
        let elapsed = started.elapsed();
        println!(
            "[{}/{total}] finished g={g} m={m} t={t} in {:.2}s (total elapsed: {:.2}s)",
            idx + 1,
            elapsed.as_secs_f64(),
            overall.elapsed().as_secs_f64(),
        );
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "normaliz exited with code {} for g={g} m={m} t={t}",
                status.code().unwrap_or(-1),
            )));
        }
    }
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
    // Detail sections live after the *** separator.
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

// ── HTML generation ───────────────────────────────────────────────────────────

const MAX_DISPLAY: usize = 40;

type GenusData = Vec<(usize, usize, usize, Vec<Vec<i64>>)>;

/// Builds the HTML section for one genus (no `<html>` wrapper).
fn build_genus_section(g: usize, data: &[(usize, usize, usize, Vec<Vec<i64>>)]) -> String {
    let count_map: HashMap<(usize, usize), usize> =
        data.iter().map(|(m, t, c, _)| ((*m, *t), *c)).collect();

    let mut h = String::new();
    let _ = writeln!(h, "<h2 id=\"g{g}\">Genus g\u{a0}=\u{a0}{g}</h2>");

    // count table
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
            let c = count_map.get(&(m, t)).copied().unwrap_or(0);
            *ct += c;
            row_sum += c;
            if c == 0 {
                h.push_str("<td class=\"zero\">\u{b7}</td>");
            } else {
                let _ = write!(
                    h,
                    "<td class=\"pos\" onclick=\"\
                     document.getElementById('sec-g{g}m{m}t{t}')\
                     .scrollIntoView({{behavior:'smooth'}})\">{c}</td>"
                );
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
    let _ = writeln!(
        h,
        "<p><strong>Total: {grand} numerical semigroup(s) of genus {g}.</strong></p>"
    );

    // detail cards
    for m in 2..=g + 1 {
        let nonempty: Vec<_> = data
            .iter()
            .filter(|&&(dm, _, c, _)| dm == m && c > 0)
            .collect();
        if nonempty.is_empty() {
            continue;
        }
        let _ = writeln!(h, "<h3>m\u{a0}=\u{a0}{m}</h3>");
        for &&(_, t, count, ref pts) in &nonempty {
            let w1 = m * t + 1;
            let selmer = m * g + m * (m - 1) / 2;
            let dim = m - 1;
            let _ = write!(
                h,
                "<details id=\"sec-g{g}m{m}t{t}\">\
                 <summary>t\u{a0}=\u{a0}{t} &nbsp;|\u{a0}\
                 w<sub>1</sub>\u{a0}=\u{a0}{w1} &nbsp;|\u{a0}\
                 \u{2211}w<sub>i</sub>\u{a0}=\u{a0}{selmer} &nbsp;|\u{a0}\
                 <strong>{count}</strong> semigroup(s)</summary>\
                 <div class=\"card\"><table class=\"pts\"><thead><tr>"
            );
            for i in 1..=dim {
                let _ = write!(h, "<th>w<sub>{i}</sub></th>");
            }
            h.push_str("</tr></thead><tbody>");
            for row in pts.iter().take(MAX_DISPLAY) {
                h.push_str("<tr>");
                for &v in row {
                    let _ = write!(h, "<td>{v}</td>");
                }
                h.push_str("</tr>");
            }
            h.push_str("</tbody></table>");
            if count > MAX_DISPLAY {
                let _ = write!(
                    h,
                    "<p class=\"trunc\">\u{2026} {MAX_DISPLAY} of {count} shown</p>"
                );
            }
            h.push_str("</div></details>\n");
        }
    }
    h
}

/// Builds the full combined HTML page (light mode) for genera 2..=gmax.
fn build_combined_html(gmax: usize, all_data: &[(usize, GenusData)]) -> String {
    let mut h = String::new();
    let _ = write!(
        h,
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"UTF-8\">\n\
         <title>Numerical Semigroups g=2\u{2026}{gmax}</title>\n\
         <style>\n\
         *{{box-sizing:border-box}}\n\
         body{{background:#fff;color:#222;font-family:monospace;\
               margin:2em auto;max-width:1200px;padding:0 1.5em}}\n\
         h1,h2,h3{{color:#1a1a8e;margin:.6em 0 .3em}}\n\
         p{{margin:.3em 0 .8em;line-height:1.5}}\n\
         table{{border-collapse:collapse;margin:.5em 0;font-size:.87em}}\n\
         th,td{{padding:3px 10px;border:1px solid #ccc;text-align:right}}\n\
         thead th,tfoot td{{background:#f0f4f8;color:#333}}\n\
         th.lbl{{text-align:left;color:#333}}\n\
         td.zero{{color:#ccc}}\n\
         td.pos{{background:#e3f2fd;color:#1565c0;font-weight:bold;cursor:pointer}}\n\
         td.pos:hover{{background:#1565c0;color:#fff}}\n\
         td.sum{{color:#1a1a8e;font-weight:bold}}\n\
         .card{{background:#f8f9fa;border-radius:6px;padding:10px 14px;margin:4px 0 8px}}\n\
         details{{margin:3px 0}}\n\
         details>summary{{cursor:pointer;color:#1565c0;list-style:none;padding:4px 2px}}\n\
         details>summary::before{{content:\"\\25b6 \";font-size:.8em}}\n\
         details[open]>summary::before{{content:\"\\25bc \";font-size:.8em}}\n\
         details>summary:hover{{color:#c00}}\n\
         .pts th{{background:#f0f4f8;color:#333}}\n\
         .pts td{{padding:1px 8px;font-size:.83em}}\n\
         .trunc{{color:#777;font-style:italic;font-size:.8em;margin:.3em 0 0}}\n\
         hr{{border:none;border-top:2px solid #ddd;margin:2em 0}}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         <h1>Numerical Semigroups \u{2014} genus 2 to {gmax}</h1>\n\
         <p>Each polytope is cut from the Kunz cone by fixing multiplicity <em>m</em>,\n\
         first Ap\u{e9}ry element w<sub>1</sub>&nbsp;=&nbsp;mt+1, and Selmer sum\n\
         \u{2211}w<sub>i</sub>&nbsp;=&nbsp;mg+m(m\u{2212}1)/2.\n\
         Each lattice point is one numerical semigroup with those parameters.</p>\n"
    );

    // grand summary table
    h.push_str(
        "<h2>Total semigroups per genus</h2>\n\
         <table><thead><tr><th class=\"lbl\">g</th>\
         <th>N(g)</th></tr></thead><tbody>",
    );
    let mut grand_total = 0usize;
    for (g, data) in all_data {
        let total: usize = data.iter().map(|(_, _, c, _)| c).sum();
        grand_total += total;
        let _ = writeln!(
            h,
            "<tr><td class=\"lbl\"><a href=\"#g{g}\">{g}</a></td>\
             <td class=\"sum\">{total}</td></tr>"
        );
    }
    let _ = writeln!(
        h,
        "</tbody><tfoot><tr><th class=\"lbl\">Total</th>\
         <td class=\"sum\">{grand_total}</td></tr></tfoot></table>"
    );

    // per-genus sections
    for (g, data) in all_data {
        h.push_str("<hr>\n");
        h.push_str(&build_genus_section(*g, data));
    }

    h.push_str("</body></html>\n");
    h
}

fn write_combined_html(gmax: usize) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    let mut all_data: Vec<(usize, GenusData)> = Vec::new();
    for g in 2..=gmax {
        let mut data: GenusData = Vec::new();
        for m in 2..=g + 1 {
            for t in 1..=g {
                let path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.out"));
                match parse_out_file(&path) {
                    Ok((count, points)) => data.push((m, t, count, points)),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e),
                }
            }
        }
        all_data.push((g, data));
    }
    let html = build_combined_html(gmax, &all_data);
    let out = dir.join(format!("semigroup_g_from2to{gmax}.html"));
    fs::write(&out, html.as_bytes())?;
    println!("wrote normaliz/semigroup_g_from2to{gmax}.html");
    Ok(())
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let gmax: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    for g in 2..=gmax {
        write_normaliz_files(g).expect("failed to run Normaliz");
    }
    write_combined_html(gmax).expect("failed to write HTML summary");
    println!("done: results in normaliz/semigroup_g_from2to{gmax}.html");
}
