//! CLI: write Normaliz input files for the Kunz-cone pair-relations matrices
//! sliced by genus `g`, multiplicity `m`, and Apéry-class parameter `t`,
//! then invoke the `normaliz` binary to produce the corresponding `.out` files,
//! and finally write a self-contained HTML summary to `normaliz/index{g}.html`.
//!
//! For each `m ∈ 2..=(g+1)` and `t ∈ 1..=g`, writes
//! `./normaliz/normaliz_g{g}_m{m}_t{t}.in` with:
//!
//! - The pair-relations inequalities `(U_i + U_j − U_{i+j})/m ≥ 0` (Kunz cone)
//! - Two affine equalities (Normaliz `inhom_equations` format, row = `[a b]` means `ax + b = 0`):
//!   - `w₁ = mt+1` (first Apéry element in class 1 mod m)
//!   - `∑wᵢ = mg + m(m−1)/2` (Selmer: pins genus to exactly g)
//!
//! The lattice points of the resulting polytope correspond bijectively to
//! numerical semigroups with genus `g`, multiplicity `m`, and `w₁ = mt+1`.
//!
//! `t` starts at 1 because `w₁ ≡ 1 (mod m)` and `w₁ ≥ m+1` for any
//! numerical semigroup with `m ≥ 2` (since `1 ∉ S` when the multiplicity is m ≥ 2).
//! The upper bound `t ≤ g` follows from `w₁ ≤ ∑wᵢ − ∑_{i=2}^{m−1} i = mg+1`
//! (using `wᵢ ≥ i` for every Apéry element).
//!
//! All `(m, t)` pairs are processed in parallel via rayon.
//!
//! Usage: `cargo run --bin normaliz [g]`  (g defaults to 10)

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use f3m::math::matrix::u_pair_relations;
use rayon::prelude::*;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::process::Command;

// ── Normaliz file generation ──────────────────────────────────────────────────

fn write_normaliz_files(g: usize) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    fs::create_dir_all(dir)?;

    // Collect all (m, t) pairs so rayon can parallelise them.
    // t=0 (w₁=1) is omitted: 1 ∈ S forces m=1, which is out of scope.
    let pairs: Vec<(usize, usize)> = (2..=g + 1)
        .flat_map(|m| (1..=g).map(move |t| (m, t)))
        .collect();

    pairs.into_par_iter().try_for_each(|(m, t)| {
        let n = m - 1; // ambient dimension = number of Kunz coordinates
        let nrows = n * (n + 1) / 2; // number of pair-relations inequalities
        let mat = u_pair_relations(m);
        let data = mat.as_slice();

        let mut buf = String::new();
        let _ = writeln!(buf, "amb_space {n}");
        let _ = writeln!(buf, "inequalities {nrows}");
        for r in 0..nrows {
            let row: Vec<String> = (0..n).map(|c| data[r * n + c].to_string()).collect();
            let _ = writeln!(buf, "{}", row.join(" "));
        }

        // Two affine equalities cut the Kunz cone to a bounded polytope.
        // Normaliz inhom_equations row format: [a₁ … aₙ b] means a·x + b = 0.
        let selmer = m * g + m * (m - 1) / 2; // ∑wᵢ for genus g (Selmer's formula)
        let w1 = m * t + 1; // first Apéry element

        let _ = writeln!(buf, "inhom_equations 2");

        // Equation 1: w₁ = w1  →  [1, 0, …, 0, −w1]
        let mut eq1 = vec![0_i64; n + 1];
        eq1[0] = 1;
        // ALLOW: w1 = mt+1 ≤ mg+1 for any reasonable genus — well within i64::MAX
        #[allow(clippy::cast_possible_wrap)]
        {
            eq1[n] = -(w1 as i64);
        }
        let _ = writeln!(
            buf,
            "{}",
            eq1.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );

        // Equation 2: ∑wᵢ = selmer  →  [1, 1, …, 1, −selmer]
        let mut eq2 = vec![1_i64; n + 1];
        // ALLOW: selmer = mg+m(m-1)/2 is genus-scale, far below i64::MAX
        #[allow(clippy::cast_possible_wrap)]
        {
            eq2[n] = -(selmer as i64);
        }
        let _ = writeln!(
            buf,
            "{}",
            eq2.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        );

        let in_path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.in"));
        fs::write(&in_path, &buf)?;

        let status = Command::new("normaliz").arg(&in_path).status()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "normaliz exited with code {} for g={g} m={m} t={t}",
                status.code().unwrap_or(-1),
            )));
        }
        Ok(())
    })
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
                let n = row.len().saturating_sub(1); // drop the trailing dehom = 1
                points.push(row[..n].to_vec());
            }
        }
    }
    Ok((count, points))
}

// ── HTML generation ───────────────────────────────────────────────────────────

const MAX_DISPLAY: usize = 40;

// ALLOW: single-responsibility HTML page builder — splitting into smaller helpers
// would obscure the visual correspondence between code and rendered page structure.
#[allow(clippy::too_many_lines)]
fn build_html(g: usize, data: &[(usize, usize, usize, Vec<Vec<i64>>)]) -> String {
    use std::fmt::Write as _;
    let mut h = String::new();

    let count_of = |m: usize, t: usize| -> usize {
        data.iter()
            .find(|&&(dm, dt, ..)| dm == m && dt == t)
            .map_or(0, |&(_, _, c, _)| c)
    };

    let _ = write!(
        h,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Kunz slices g={g}</title>
<style>
*{{box-sizing:border-box}}
body{{background:#1a1a2e;color:#e0e0e0;font-family:monospace;
      margin:2em auto;max-width:1200px;padding:0 1.5em}}
h1,h2,h3{{color:#e94560;margin:.6em 0 .3em}}
p{{margin:.3em 0 .8em;line-height:1.5}}
table{{border-collapse:collapse;margin:.5em 0;font-size:.87em}}
th,td{{padding:3px 10px;border:1px solid #2a2a4a;text-align:right}}
thead th,tfoot td{{background:#16213e;color:#a8dadc}}
th.lbl{{text-align:left;color:#a8dadc}}
td.zero{{color:#3a3a5a}}
td.pos{{background:#0f3460;color:#e94560;font-weight:bold;cursor:pointer}}
td.pos:hover{{background:#e94560;color:#fff}}
td.sum{{color:#a8dadc;font-weight:bold}}
.card{{background:#16213e;border-radius:6px;padding:10px 14px;margin:4px 0 8px}}
details{{margin:3px 0}}
details>summary{{cursor:pointer;color:#a8dadc;list-style:none;padding:4px 2px}}
details>summary::before{{content:"▶ ";font-size:.8em}}
details[open]>summary::before{{content:"▼ ";font-size:.8em}}
details>summary:hover{{color:#e94560}}
.pts th{{background:#0d1b2a;color:#a8dadc}}
.pts td{{padding:1px 8px;font-size:.83em}}
.trunc{{color:#777;font-style:italic;font-size:.8em;margin:.3em 0 0}}
</style>
</head>
<body>
<h1>Kunz cone slices — genus g = {g}</h1>
<p>Each polytope is cut from the Kunz cone by fixing multiplicity <em>m</em>,
first Apéry element w<sub>1</sub>&nbsp;=&nbsp;mt+1, and Selmer sum
∑w<sub>i</sub>&nbsp;=&nbsp;mg+m(m−1)/2.
Each lattice point is one numerical semigroup with those parameters.<br>
Click a highlighted count to jump to its detail.</p>
"#
    );

    // ── count table ───────────────────────────────────────────────────────────
    h.push_str("<h2>Count table</h2>\n<table>\n<thead><tr><th class=\"lbl\">m \\ t</th>");
    for t in 1..=g {
        let _ = write!(h, "<th>{t}</th>");
    }
    h.push_str("<th class=\"sum\">Σ</th></tr></thead>\n<tbody>");

    // col_totals is 1-indexed (slot 0 unused) to keep t values natural.
    let mut col_totals = vec![0usize; g + 1];
    let mut grand = 0usize;
    for m in 2..=g + 1 {
        let _ = write!(h, "\n<tr><th class=\"lbl\">m = {m}</th>");
        let mut row_sum = 0usize;
        for (t, ct) in col_totals.iter_mut().enumerate().skip(1).take(g) {
            let c = count_of(m, t);
            *ct += c;
            row_sum += c;
            if c == 0 {
                h.push_str("<td class=\"zero\">·</td>");
            } else {
                let _ = write!(
                    h,
                    "<td class=\"pos\" onclick=\"\
                     document.getElementById('sec-m{m}t{t}').scrollIntoView({{behavior:'smooth'}})\">\
                     {c}</td>"
                );
            }
        }
        grand += row_sum;
        let _ = write!(h, "<td class=\"sum\">{row_sum}</td></tr>");
    }

    h.push_str("\n</tbody>\n<tfoot><tr><th class=\"lbl\">Σ</th>");
    for ct in col_totals.iter().skip(1) {
        let _ = write!(h, "<td class=\"sum\">{ct}</td>");
    }
    let _ = write!(h, "<td class=\"sum\">{grand}</td></tr></tfoot>\n</table>\n");

    // ── detail cards ──────────────────────────────────────────────────────────
    h.push_str("<h2>Details</h2>\n");
    for m in 2..=g + 1 {
        let nonempty: Vec<_> = data
            .iter()
            .filter(|&&(dm, _, c, _)| dm == m && c > 0)
            .collect();
        if nonempty.is_empty() {
            continue;
        }
        let _ = writeln!(h, "<h3>m = {m}</h3>");
        for &&(_, t, count, ref pts) in &nonempty {
            let w1 = m * t + 1;
            let selmer = m * g + m * (m - 1) / 2;
            let dim = m - 1;
            let _ = write!(
                h,
                "<details id=\"sec-m{m}t{t}\">\
                 <summary>t = {t} &nbsp;|&nbsp; \
                 w<sub>1</sub> = {w1} &nbsp;|&nbsp; \
                 ∑w<sub>i</sub> = {selmer} &nbsp;|&nbsp; \
                 <strong>{count}</strong> semigroup(s)</summary>\
                 <div class=\"card\">\
                 <table class=\"pts\"><thead><tr>"
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
                let _ = write!(h, "<p class=\"trunc\">… {MAX_DISPLAY} of {count} shown</p>");
            }
            h.push_str("</div></details>\n");
        }
    }

    h.push_str("</body></html>\n");
    h
}

/// Reads all `normaliz_g{g}_m*_t*.out` files in `./normaliz/`, parses them,
/// and writes a self-contained HTML summary to `./normaliz/index{g}.html`.
fn write_index_html(g: usize) -> std::io::Result<()> {
    let dir = Path::new("normaliz");
    let mut data: Vec<(usize, usize, usize, Vec<Vec<i64>>)> = Vec::new();
    for m in 2..=g + 1 {
        for t in 1..=g {
            let path = dir.join(format!("normaliz_g{g}_m{m}_t{t}.out"));
            if path.exists() {
                let (count, points) = parse_out_file(&path)?;
                data.push((m, t, count, points));
            }
        }
    }
    let html = build_html(g, &data);
    let out = dir.join(format!("index{g}.html"));
    fs::write(&out, html.as_bytes())?;
    println!("wrote normaliz/index{g}.html");
    Ok(())
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let g: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    write_normaliz_files(g).expect("failed to run Normaliz");
    write_index_html(g).expect("failed to write HTML summary");
    println!("done: up to {} input/output pairs in normaliz/", g * g);
}
