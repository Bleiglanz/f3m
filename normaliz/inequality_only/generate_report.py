#!/usr/bin/env python3
"""Parse Normaliz .out files and generate an HTML summary report.

Usage:
    python3 generate_report.py          # from the normaliz/ directory
    python3 generate_report.py <dir>    # pass the directory explicitly
Writes index.html to the same directory as the .out files.
"""
import re
import sys
from pathlib import Path

MAX_DISPLAY = 50  # max Hilbert-basis rows shown in the detail table


# ── Parsing ──────────────────────────────────────────────────────────────────

def _int_row(line: str) -> list[int] | None:
    s = line.strip()
    if not s:
        return None
    try:
        return list(map(int, s.split()))
    except ValueError:
        return None


def parse_file(path: Path) -> dict:
    lines = path.read_text(encoding="utf-8").splitlines()

    # Split at the first *** separator into header / detail
    sep = next((i for i, l in enumerate(lines) if l.startswith("***")), None)
    header_text = "\n".join(lines[:sep] if sep is not None else lines)
    detail = lines[sep + 1:] if sep is not None else []

    def first_int(pat: str, text: str = header_text) -> int | None:
        m = re.search(pat, text, re.MULTILINE)
        return int(m.group(1)) if m else None

    d: dict = {}
    d["hb_count"] = first_int(r"^(\d+) Hilbert basis elements")
    d["er_count"] = first_int(r"^(\d+) extreme rays")
    d["sh_count"] = first_int(r"^(\d+) support hyperplanes")
    d["emb_dim"]  = first_int(r"embedding dimension = (\d+)")
    d["rank_"]    = first_int(r"rank = (\d+)")
    d["mult"]     = first_int(r"^multiplicity = (\d+)")
    d["cg_rank"]  = first_int(r"rank of class group = (\d+)")
    d["hs_deg"]   = first_int(r"degree of Hilbert Series as rational function = (-?\d+)")
    d["vol"]      = first_int(r"resulting sum of \|det\|s = (\d+)")

    d["symmetric"] = "The numerator of the Hilbert series is symmetric." in header_text
    d["cg_free"]   = "class group is free" in header_text
    d["graded"]    = "No implicit grading found" not in header_text

    m2 = re.search(r"Hilbert series:\n([ \d]+)", header_text)
    d["hs_numer"] = m2.group(1).strip() if m2 else None

    m3 = re.search(r"Hilbert polynomial:\n([ \d]+)", header_text)
    d["hp_num"] = m3.group(1).strip() if m3 else None
    m4 = re.search(r"with common denominator = (\d+)", header_text)
    d["hp_den"] = int(m4.group(1)) if m4 else None

    # Parse detail sections line-by-line
    SECT = re.compile(r"^(\d+) (.+):$")
    d["hb_rows"] = []
    d["er_rows"] = []
    d["sh_rows"] = []

    i = 0
    while i < len(detail):
        line = detail[i].strip()
        i += 1
        if not line or line.startswith("***"):
            continue
        mm = SECT.match(line)
        if not mm:
            continue

        count = int(mm.group(1))
        label = mm.group(2).lower()

        # Read exactly `count` integer rows (skip blank lines, stop at non-int text)
        rows: list[list[int]] = []
        while i < len(detail) and len(rows) < count:
            r = _int_row(detail[i])
            if r is None:
                if detail[i].strip():
                    break       # non-blank, non-integer → end of section
                i += 1
                continue
            rows.append(r)
            i += 1

        lbl = label
        if "lattice points" in lbl or ("hilbert basis" in lbl and "further" not in lbl):
            d["hb_rows"] = rows[:MAX_DISPLAY]
        elif "extreme ray" in lbl:
            d["er_rows"] = rows[:MAX_DISPLAY]
        elif "support hyperplane" in lbl:
            d["sh_rows"] = rows           # support hyperplanes are small; keep all

    # For m=14-style files where Normaliz only computed extreme rays (no HB)
    d["hb_total"]  = d["hb_count"] if d["hb_count"] is not None else d["er_count"]
    d["hb_is_er"]  = d["hb_count"] is None   # using ER rows as HB proxy
    return d


# ── HTML helpers ──────────────────────────────────────────────────────────────

def esc(s: str) -> str:
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")

def fmtn(v: int | None) -> str:
    return f"{v:,}" if v is not None else "—"

def render_mat(rows: list[list[int]], total: int | None = None) -> str:
    if not rows:
        return '<em style="color:var(--muted)">—</em>'
    n_cols = len(rows[0])
    trs = "\n".join(
        "<tr>" + "".join(f"<td>{x}</td>" for x in r) + "</tr>" for r in rows
    )
    trunc = ""
    if total is not None and total > len(rows):
        trunc = (
            f'<tr><td colspan="{n_cols}" class="trunc">'
            f"… {total - len(rows):,} more rows not shown"
            f"</td></tr>"
        )
    return (
        '<div class="mat-wrap">'
        '<table class="matrix"><tbody>'
        + trs + trunc +
        "</tbody></table></div>"
    )


# ── CSS ───────────────────────────────────────────────────────────────────────

CSS = """\
:root{
  --bg:#0d1117;--surface:#161b22;--s2:#21262d;--border:#30363d;
  --text:#e6edf3;--muted:#7d8590;--accent:#58a6ff;--accent2:#d2a8ff;
  --yes:#3fb950;--no:#f85149;--mono:"JetBrains Mono","Fira Code",monospace;
}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,sans-serif;background:var(--bg);color:var(--text);
     line-height:1.6;padding:2rem 1rem;max-width:1400px;margin:0 auto}
h1{font-size:1.8rem;color:var(--accent)}
.sub{color:var(--muted);margin:.3rem 0 2rem;font-size:.9rem}
h2{font-size:1.2rem;color:var(--accent2);margin-bottom:.7rem}
a{color:var(--accent);text-decoration:none}
a:hover{text-decoration:underline}

/* Summary table */
.tw{overflow-x:auto;margin-bottom:2.5rem}
table.s{border-collapse:collapse;width:100%;font-size:.88rem}
table.s th{
  background:var(--s2);color:var(--muted);font-weight:600;
  padding:.5rem .75rem;border-bottom:2px solid var(--border);
  text-align:right;white-space:nowrap}
table.s th:first-child{text-align:left}
table.s td{padding:.35rem .75rem;border-bottom:1px solid var(--border);text-align:right}
table.s td:first-child{text-align:left;font-weight:600}
table.s tr:hover td{background:var(--surface)}
.yes{color:var(--yes)}.no{color:var(--no)}
.mono{font-family:var(--mono);font-size:.78rem;text-align:left!important}

/* TOC nav */
nav.toc{
  background:var(--surface);border:1px solid var(--border);border-radius:8px;
  padding:.7rem 1rem;margin-bottom:1.5rem;
  display:flex;flex-wrap:wrap;gap:.4rem;align-items:center}
.toc-lbl{color:var(--muted);font-size:.82rem;margin-right:.25rem}
nav.toc a{
  background:var(--s2);border:1px solid var(--border);border-radius:4px;
  padding:.12rem .55rem;font-size:.82rem}
nav.toc a:hover{background:var(--border)}

/* Detail card */
.card{
  background:var(--surface);border:1px solid var(--border);border-radius:8px;
  padding:1.4rem;margin-bottom:1.25rem;scroll-margin-top:1rem}
.kv-grid{
  display:grid;grid-template-columns:repeat(auto-fill,minmax(125px,1fr));
  gap:.55rem;margin-bottom:.9rem}
.kv{
  background:var(--s2);border:1px solid var(--border);border-radius:6px;
  padding:.5rem .65rem;display:flex;flex-direction:column;align-items:center}
.kv-v{font-size:1.2rem;font-weight:700;color:var(--accent)}
.kv-l{font-size:.68rem;color:var(--muted);margin-top:1px;text-align:center}
dl.p{display:grid;grid-template-columns:max-content 1fr;
     gap:.2rem .85rem;font-size:.88rem;margin-bottom:.7rem}
dt{color:var(--muted)}
dd.m{font-family:var(--mono)}

/* Collapsible matrix */
details.d{
  margin-top:.55rem;background:var(--bg);border:1px solid var(--border);
  border-radius:6px;padding:.35rem .7rem}
details.d>summary{
  cursor:pointer;font-size:.83rem;color:var(--muted);
  user-select:none;padding:.2rem 0}
details.d>summary:hover,details.d[open]>summary{color:var(--text)}
details.d[open]>summary{margin-bottom:.4rem}
.mat-wrap{overflow-x:auto}
table.matrix{border-collapse:collapse;font-family:var(--mono);font-size:.75rem}
table.matrix td{padding:1px 5px;text-align:right}
table.matrix tr:nth-child(even) td{background:rgba(255,255,255,.025)}
.trunc{text-align:center!important;color:var(--muted);font-style:italic;padding:.3rem}
"""


# ── HTML builder ──────────────────────────────────────────────────────────────

def build_html(records: list[dict]) -> str:
    # ── Summary table ──
    trs = []
    for d in records:
        m = d["m"]
        symm_td = '<td class="yes">✓</td>' if d["symmetric"] else '<td class="no">✗</td>'
        hs_td = (
            f'<td class="mono">{esc(d["hs_numer"])}</td>' if d["hs_numer"]
            else "<td>—</td>"
        )
        trs.append(
            "<tr>"
            f'<td><a href="#m{m}">m = {m}</a></td>'
            f"<td>{d['emb_dim'] or '—'}</td>"
            f"<td>{fmtn(d['hb_total'])}</td>"
            f"<td>{fmtn(d['er_count'])}</td>"
            f"<td>{d['sh_count'] or '—'}</td>"
            f"<td>{d['mult'] if d['mult'] is not None else '—'}</td>"
            + symm_td
            + f"<td>{d['cg_rank'] if d['cg_rank'] is not None else '—'}</td>"
            + hs_td
            + "</tr>"
        )

    # ── Detail cards ──
    cards = []
    for d in records:
        m = d["m"]
        hb_label = (
            "Hilbert basis" if not d["hb_is_er"]
            else "Extreme rays (HB not computed)"
        )
        hb_total = d["hb_total"]
        hb_display = d["hb_rows"] if d["hb_rows"] else d["er_rows"]

        stat_items = [
            (fmtn(hb_total),       "Hilbert basis"),
            (fmtn(d["er_count"]),  "Extreme rays"),
            (str(d["sh_count"]) if d["sh_count"] else "—", "Supp. hyperplanes"),
            (str(d["emb_dim"])  if d["emb_dim"]  else "—", f"dim = m−1 = {m - 1}"),
        ]
        kvs = "".join(
            f'<div class="kv">'
            f'<span class="kv-v">{v}</span>'
            f'<span class="kv-l">{lbl}</span>'
            f"</div>"
            for v, lbl in stat_items
        )

        props = []
        if d["rank_"] is not None:
            props.append(f"<dt>Rank</dt><dd>{d['rank_']} (maximal)</dd>")
        if d["mult"] is not None:
            props.append(f"<dt>Multiplicity</dt><dd>{d['mult']}</dd>")
        color = "yes" if d["symmetric"] else "no"
        label = "Yes ✓" if d["symmetric"] else "No ✗"
        props.append(f'<dt>Gorenstein?</dt><dd style="color:var(--{color})">{label}</dd>')
        if d["hs_numer"]:
            props.append(
                f'<dt>Hilbert series num.</dt><dd class="m">[{esc(d["hs_numer"])}]</dd>'
            )
        if d["hs_deg"] is not None:
            props.append(f"<dt>HS degree (rat. fn.)</dt><dd>{d['hs_deg']}</dd>")
        if d["hp_num"]:
            denom = f" / {d['hp_den']}" if d["hp_den"] and d["hp_den"] != 1 else ""
            props.append(
                f'<dt>Hilbert polynomial</dt><dd class="m">[{esc(d["hp_num"])}]{denom}</dd>'
            )
        if d["vol"] is not None:
            props.append(f"<dt>∑|det| (triangul. vol)</dt><dd>{d['vol']:,}</dd>")
        if d["cg_rank"] is not None:
            free = ", free" if d["cg_free"] else ""
            props.append(f"<dt>Class group rank</dt><dd>{d['cg_rank']}{free}</dd>")

        trunc_note = ""
        if hb_total and len(hb_display) < hb_total:
            trunc_note = f" (first {MAX_DISPLAY} shown)"

        hb_mat = render_mat(hb_display, total=hb_total)
        sh_mat = render_mat(d["sh_rows"])

        cards.append(
            f'<section id="m{m}" class="card">'
            f"<h2>m = {m}</h2>"
            f'<div class="kv-grid">{kvs}</div>'
            f'<dl class="p">{"".join(props)}</dl>'
            f'<details class="d"><summary>'
            f"{hb_label} ({fmtn(hb_total)}){trunc_note}"
            f"</summary>{hb_mat}</details>"
            f'<details class="d"><summary>'
            f"Support hyperplanes ({d['sh_count'] or '?'})"
            f"</summary>{sh_mat}</details>"
            f"</section>"
        )

    toc_links = " ".join(f'<a href="#m{d["m"]}">m={d["m"]}</a>' for d in records)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Kunz Cone — Normaliz m = 2…14</title>
  <style>{CSS}</style>
</head>
<body>
  <h1>Kunz Cone — Normaliz Results</h1>
  <p class="sub">
    Hilbert basis &amp; geometry of the Kunz cone C(m) for m = 2 … 14.<br>
    Inequalities: (U<sub>i</sub> + U<sub>j</sub> − U<sub>i+j</sub>) / m ≥ 0 &nbsp;·&nbsp;
    Computed by <a href="https://www.normaliz.uni-osnabrueck.de/" target="_blank">Normaliz</a>.
  </p>

  <h2>Summary</h2>
  <div class="tw">
    <table class="s">
      <thead><tr>
        <th>m</th><th>dim</th><th># HB</th><th># Extr. rays</th>
        <th># Supp. hyp.</th><th>Multiplicity</th>
        <th>Gorenstein?</th><th>cl(G) rank</th>
        <th>Hilbert series numerator</th>
      </tr></thead>
      <tbody>{"".join(trs)}</tbody>
    </table>
  </div>

  <nav class="toc">
    <span class="toc-lbl">Jump to:</span> {toc_links}
  </nav>

  {"".join(cards)}
</body>
</html>"""


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    directory = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    files = sorted(
        directory.glob("normaliz_*.out"),
        key=lambda p: int(re.search(r"(\d+)", p.stem).group(1)),
    )
    if not files:
        print(f"No normaliz_*.out files found in {directory}", file=sys.stderr)
        sys.exit(1)

    records = []
    for f in files:
        m_val = int(re.search(r"(\d+)", f.stem).group(1))
        print(f"  parsing m={m_val} …")
        d = parse_file(f)
        d["m"] = m_val
        records.append(d)

    html = build_html(records)
    out = directory / "index.html"
    out.write_text(html, encoding="utf-8")
    print(f"\n  → {out}  ({len(html):,} bytes)")


if __name__ == "__main__":
    main()
