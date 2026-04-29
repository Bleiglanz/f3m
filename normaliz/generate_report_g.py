#!/usr/bin/env python3
"""Generate index{g}.html from existing normaliz_g{g}_m*_t*.out files.

Usage:
    python3 generate_report_g.py [g]     (default g=5)
    python3 generate_report_g.py 5

Reads  ./normaliz_g{g}_m{m}_t{t}.out  for all valid (m, t) pairs and writes
./index{g}.html.  Run from inside the normaliz/ directory.
"""

import sys
import re
import os

MAX_DISPLAY = 40

# ── parser ─────────────────────────────────────────────────────────────────────

def parse_out(path):
    """Return (count, points) for a Normaliz .out file.

    count  – number of lattice points (0 for infeasible polytopes)
    points – list of coordinate tuples, trailing dehomogenization column removed
    """
    try:
        text = open(path).read()
    except FileNotFoundError:
        return 0, []

    m = re.match(r"^\s*(\d+)\s+lattice points in polytope", text)
    count = int(m.group(1)) if m else 0
    if count == 0:
        return 0, []

    sep = text.find("***")
    after = text[sep:] if sep >= 0 else ""
    marker = f"{count} lattice points in polytope (module generators):"
    pos = after.find(marker)
    if pos < 0:
        return count, []

    points = []
    for line in after[pos:].splitlines()[1:count + 1]:
        nums = list(map(int, line.split()))
        if nums:
            points.append(tuple(nums[:-1]))   # drop last dehom column
    return count, points

# ── HTML builder ───────────────────────────────────────────────────────────────

CSS = """
*{box-sizing:border-box}
body{background:#1a1a2e;color:#e0e0e0;font-family:monospace;
     margin:2em auto;max-width:1200px;padding:0 1.5em}
h1,h2,h3{color:#e94560;margin:.6em 0 .3em}
p{margin:.3em 0 .8em;line-height:1.5}
table{border-collapse:collapse;margin:.5em 0;font-size:.87em}
th,td{padding:3px 10px;border:1px solid #2a2a4a;text-align:right}
thead th,tfoot td{background:#16213e;color:#a8dadc}
th.lbl{text-align:left;color:#a8dadc}
td.zero{color:#3a3a5a}
td.pos{background:#0f3460;color:#e94560;font-weight:bold;cursor:pointer}
td.pos:hover{background:#e94560;color:#fff}
td.sum{color:#a8dadc;font-weight:bold}
.card{background:#16213e;border-radius:6px;padding:10px 14px;margin:4px 0 8px}
details{margin:3px 0}
details>summary{cursor:pointer;color:#a8dadc;list-style:none;padding:4px 2px}
details>summary::before{content:"▶ ";font-size:.8em}
details[open]>summary::before{content:"▼ ";font-size:.8em}
details>summary:hover{color:#e94560}
.pts th{background:#0d1b2a;color:#a8dadc}
.pts td{padding:1px 8px;font-size:.83em}
.trunc{color:#777;font-style:italic;font-size:.8em;margin:.3em 0 0}
"""

def build_html(g, data):
    """Build and return the full HTML page as a string.

    data – list of (m, t, count, points) tuples, all (m, t) pairs included.
    """
    idx = {(m, t): (count, pts) for m, t, count, pts in data}

    lines = []
    w = lines.append

    w(f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Kunz slices g={g}</title>
<style>{CSS}</style>
</head>
<body>
<h1>Kunz cone slices &#8212; genus g = {g}</h1>
<p>Each polytope is cut from the Kunz cone by fixing multiplicity <em>m</em>,
first Ap&#233;ry element w<sub>1</sub>&nbsp;=&nbsp;mt+1, and Selmer sum
&#8721;w<sub>i</sub>&nbsp;=&nbsp;mg+m(m&#8722;1)/2.
Each lattice point is one numerical semigroup with those parameters.<br>
Click a highlighted count to jump to its detail.</p>""")

    # ── count table ────────────────────────────────────────────────────────────
    w("<h2>Count table</h2>")
    w("<table><thead><tr><th class='lbl'>m \\ t</th>")
    for t in range(1, g + 1):
        w(f"<th>{t}</th>")
    w("<th class='sum'>&#931;</th></tr></thead><tbody>")

    col_totals = [0] * (g + 1)
    grand = 0
    for m in range(2, g + 2):
        w(f"\n<tr><th class='lbl'>m = {m}</th>")
        row_sum = 0
        for t in range(1, g + 1):
            c, _ = idx.get((m, t), (0, []))
            col_totals[t] += c
            row_sum += c
            if c == 0:
                w("<td class='zero'>·</td>")
            else:
                w(f"<td class='pos' "
                  f"onclick=\"document.getElementById('sec-m{m}t{t}')"
                  f".scrollIntoView({{behavior:'smooth'}})\">{c}</td>")
        grand += row_sum
        w(f"<td class='sum'>{row_sum}</td></tr>")

    w("\n</tbody><tfoot><tr><th class='lbl'>&#931;</th>")
    for t in range(1, g + 1):
        w(f"<td class='sum'>{col_totals[t]}</td>")
    w(f"<td class='sum'>{grand}</td></tr></tfoot></table>")

    # ── detail cards ───────────────────────────────────────────────────────────
    w("<h2>Details</h2>")
    for m in range(2, g + 2):
        nonempty = [(t, *idx[(m, t)]) for t in range(1, g + 1)
                    if (m, t) in idx and idx[(m, t)][0] > 0]
        if not nonempty:
            continue
        w(f"<h3>m = {m}</h3>")
        for t, count, pts in nonempty:
            w1 = m * t + 1
            selmer = m * g + m * (m - 1) // 2
            dim = m - 1
            w(f"<details id='sec-m{m}t{t}'>"
              f"<summary>t = {t} &nbsp;|&nbsp; "
              f"w<sub>1</sub> = {w1} &nbsp;|&nbsp; "
              f"&#8721;w<sub>i</sub> = {selmer} &nbsp;|&nbsp; "
              f"<strong>{count}</strong> semigroup(s)</summary>"
              f"<div class='card'>"
              f"<table class='pts'><thead><tr>")
            for i in range(1, dim + 1):
                w(f"<th>w<sub>{i}</sub></th>")
            w("</tr></thead><tbody>")
            for row in pts[:MAX_DISPLAY]:
                w("<tr>" + "".join(f"<td>{v}</td>" for v in row) + "</tr>")
            w("</tbody></table>")
            if count > MAX_DISPLAY:
                w(f"<p class='trunc'>&#8230; {MAX_DISPLAY} of {count} shown</p>")
            w("</div></details>")

    w("</body></html>")
    return "\n".join(lines)


# ── main ───────────────────────────────────────────────────────────────────────

def main():
    g = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)

    data = []
    for m in range(2, g + 2):
        for t in range(1, g + 1):
            path = f"normaliz_g{g}_m{m}_t{t}.out"
            count, pts = parse_out(path)
            data.append((m, t, count, pts))

    feasible = sum(1 for _, _, c, _ in data if c > 0)
    total = sum(c for _, _, c, _ in data)
    print(f"g={g}: {feasible} feasible (m,t) pairs, {total} total lattice points")

    html = build_html(g, data)
    out_path = f"index{g}.html"
    with open(out_path, "w") as f:
        f.write(html)
    print(f"wrote {script_dir}/{out_path}  ({len(html):,} bytes)")


if __name__ == "__main__":
    main()
