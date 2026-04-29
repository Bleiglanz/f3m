import init, {
  js_compute, combined_table, tilt_table, shortprop_tds, eval_expr,
  js_graph_edges_text, js_node_class, js_classify_table, js_diagonals_table, js_rolf_primes,
  state_push, state_get, state_len, state_current_idx, state_set_current_idx,
  state_get_eva_expr, state_set_eva_expr, state_gap_output, state_cmp,
  state_get_show_kunz, state_set_show_kunz,
  state_set_show_classification,
} from '../pkg/f3m.js';
import { render3d } from './view3d.js';
import { rebuildGraph, setupGraphUpto, setupShowGaps, setupGraphToggle } from './graph.js';

const PROP_THEAD_TR = '<tr><th title="Index and operation label">#</th><th title="Generator added (+) or removed (\u2212)">toggle</th><th title="Multiplicity: smallest positive element">m</th><th title="Frobenius number: largest gap">f</th><th title="Embedding dimension: number of minimal generators">e</th><th title="Genus: number of gaps">g</th><th title="Sporadic elements: elements of S below the conductor f+1">\u03C3</th><th title="Reflected gaps: gaps n where f\u2212n is also a gap">r</th><th title="Reflected Ap\u00E9ry: Ap\u00E9ry elements w where w\u2212m is a reflected gap">ra</th><th title="Fundamental gaps: gaps not expressible as sum of two smaller gaps">fg</th><th title="Type: number of pseudo-Frobenius numbers">t</th><th title="Symmetric: t=1 and g=(f+1)/2">Sym</th><th title="Minimal generators">gen</th><th title="Pseudo-Frobenius numbers: maximals of \u2124 \u2216 S">PF</th><th title="Special pseudo-Frobenius: PF that are differences of generators">SPF</th><th title="Wilf quotient: \u03C3/(f+1) \u2265 1/e (conjecture)">Wilf</th><th title="Wilf conjecture lower bound: 1/e">1/e</th><th title="Expression evaluated for this semigroup">expr</th><th title="Result of the expression">value</th><th title="Set-containment relation with previous entry">&#8838;?</th></tr>';

const PRIMES_LIST = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97];

// ── UI-only state (Rust owns the data) ────────────────────────────────────────
let currentGenSet = null;
let currentS = null;
let computing = false;
let navigating = false;
let _computeLabel = '⏎'; // label shown in # column of history
const busyBanner = document.getElementById('busy-banner');

// Display an error message in the error banner.
function showError(msg) {
  const errEl = document.getElementById('error');
  errEl.textContent = msg;
  errEl.style.display = 'block';
}

window.addEventListener('error', e => {
  if (e.message?.includes('ResizeObserver')) { return; }
  showError(`JS error: ${e.message}`);
});
window.addEventListener('unhandledrejection', e => showError(`Unhandled promise rejection: ${e.reason}`));

function setBusy(b) {
  computing = b;
  busyBanner.style.display = b ? 'block' : 'none';
}

// Flash the banner if a new action is attempted while computing; return true if busy.
function guardBusy() {
  if (!computing) { return false; }
  busyBanner.classList.add('busy-flash');
  setTimeout(() => busyBanner.classList.remove('busy-flash'), 300);
  return true;
}

// Wrap a callback so it only runs when the app is not busy.
const guarded = fn => () => { if (!guardBusy()) { fn(); } };

const gensInput = document.getElementById('gens');

// Parse a generator string: split on any non-digit sequence, keep positive integers.
const parseGens = str => str.split(/\D+/).map(t => parseInt(t, 10)).filter(n => !isNaN(n) && n > 0);

// Build a LaTeX source string describing semigroup s.
function buildLatexSource(s) {
  const gens = Array.from(s.gen_set);
  const pf = Array.from(s.pf);
  const g = s.count_gap;
  const c = s.f + 1;
  const sigma = s.count_set; // σ: elements below conductor = c − g
  const r = Array.from(s.blob).length; // reflected gap count

  const genStr = gens.join(',\\, ');
  const pfStr = pf.length ? `\\{${pf.join(',\\, ')}\\}` : '\\emptyset';

  const blocks = [];

  blocks.push(
    `S = \\langle ${genStr} \\rangle \\subseteq \\mathbb{N}_0`
  );

  blocks.push([
    `\\begin{array}{rll}`,
    `m(S) &= ${s.m}      & \\text{multiplicity (smallest positive element)} \\\\`,
    `f(S) &= ${s.f}      & \\text{Frobenius number (largest gap)} \\\\`,
    `e(S) &= ${s.e}      & \\text{embedding dimension} \\\\`,
    `g(S) &= ${g}        & \\text{genus } |\\mathbb{N}_0 \\setminus S| \\\\`,
    `c(S) &= ${c}        & \\text{conductor } (f+1) \\\\`,
    `\\sigma(S) &= ${sigma} & \\text{elements below conductor } (c - g) \\\\`,
    `r(S) &= ${r}        & \\text{reflected gaps } |\\{n : n \\notin S,\\, f{-}n \\notin S\\}| \\\\`,
    `t(S) &= ${s.type_t} & \\text{type } |\\mathrm{PF}(S)|`,
    `\\end{array}`,
  ].join('\n'));

  blocks.push(`\\mathrm{PF}(S) = ${pfStr}`);

  if (s.is_symmetric) {
    blocks.push(`S \\text{ is \\textbf{symmetric}:}\\quad t = 1,\\quad g = \\tfrac{f+1}{2} = ${g}`);
  } else {
    blocks.push(`S \\text{ is \\textbf{not symmetric}:}\\quad t(S) = ${s.type_t} \\neq 1`);
  }

  blocks.push(
    `\\text{Wilf:}\\quad \\frac{\\sigma}{c} = \\frac{${sigma}}{${c}} \\approx ${(sigma / c).toFixed(4)} \\;\\geq\\; \\frac{1}{e} = \\frac{1}{${s.e}} \\approx ${(1 / s.e).toFixed(4)}`
  );

  return blocks.join('\n\n');
}

// Render a multi-block LaTeX source into #latex-rendered using KaTeX.
// Blocks are separated by blank lines; each is rendered in display mode.
function renderLatexPreview(source) {
  const el = document.getElementById('latex-rendered');
  if (!window.katex) { el.textContent = 'KaTeX not loaded'; return; }
  el.innerHTML = source.split(/\n\n+/)
    .filter(b => b.trim())
    .map(b => {
      try {
        return window.katex.renderToString(b.trim(), { displayMode: true, throwOnError: false });
      } catch (err) {
        return `<span class="latex-error">${err.message}</span>`;
      }
    })
    .join('');
}

function buildLatex(s) {
  const src = buildLatexSource(s);
  document.getElementById('latex-source').value = src;
  renderLatexPreview(src);
}

// Convert a cell's text content to a CSV-safe field (quote if needed).
function csvField(text) {
  const s = text.replace(/\s+/g, ' ').trim();
  return (s.includes(',') || s.includes('"') || s.includes('\n')) ? `"${s.replace(/"/g, '""')}"` : s;
}

// Extract visible text from a table cell, excluding hidden popup content.
function cellText(td) {
  const count = td.querySelector('.popup-count');
  return count ? count.textContent : td.textContent;
}

// Build CSV text from the current history table and write it to #csv-output.
function buildCsv() {
  const CSV_HEADER = '#,toggle,m,f,e,g,σ,r,ra,fg,t,Sym,gen,PF,SPF,Wilf,1/e,expr,value,⊆?,generators';
  const rows = Array.from(document.querySelectorAll('#history-tbody tr'));
  const lines = rows.map(tr => {
    const cells = Array.from(tr.cells).map(td => csvField(cellText(td))).join(',');
    const gens = csvField(tr.dataset.gens ?? '');
    return `${cells},${gens}`;
  });
  document.getElementById('csv-output').value = [CSV_HEADER, ...lines].join('\n');
}

// Build a history table row for semigroup `s` at index `idx`.
const sIdx = n => `S<sub>${n}</sub>`;

function historyRow(s, idx, label, toggle, expr, value, cmp) {
  const valStr = value ?? '—';
  const toggleStr = toggle
    ? `${sIdx(toggle.from)}${toggle.sign}<span class="${toggle.cls}">${toggle.n}</span>`
    : '—';
  const gens = Array.from(s.gen_set).join(' ');
  return `<tr class="history-row" data-idx="${idx}" data-gens="${gens}"><td>${idx}:${label}</td><td>${toggleStr}</td>${shortprop_tds(s)}<td class="left">${expr}</td><td>${valStr}</td><td>${cmp}</td></tr>`;
}

// Render a semigroup: update all UI tabs. Caller must call state_push first.
function render(s, toggle = null, label = '⏎') {
  currentGenSet = Array.from(s.gen_set);
  currentS = s;
  gensInput.value = currentGenSet.join(', ');
  const idx = state_current_idx();
  const expr = state_get_eva_expr();
  const exprVal = eval_expr(expr, s);
  const cmpSourceIdx = toggle ? toggle.from : (idx > 0 ? idx - 1 : null);
  const cmp = cmpSourceIdx !== null
    ? `${sIdx(idx)}&nbsp;${state_cmp(idx, cmpSourceIdx)}&nbsp;${sIdx(cmpSourceIdx)}`
    : '—';
  const rowHtml = historyRow(s, idx, label, toggle, expr, exprVal, cmp);

  // History tab — skip append when restoring via back/forward
  if (!navigating) {
    const histTbody = document.getElementById('history-tbody');
    if (idx > 0 && idx % 10 === 0) {
      histTbody.insertAdjacentHTML('beforeend', PROP_THEAD_TR);
    }
    histTbody.insertAdjacentHTML('beforeend', rowHtml);
    document.getElementById('history-gap').textContent = state_gap_output();
  }

  // Top property row + graph tab
  document.getElementById('current-prop-tbody').innerHTML = rowHtml;
  const graphUpto = document.getElementById('graph-upto');
  graphUpto.min = s.m;
  graphUpto.max = s.f + s.m;
  graphUpto.value = s.m;
  document.getElementById('graph-upto-val').textContent = s.m;
  document.getElementById('graph-edges-text').value = js_graph_edges_text(s, s.m);
  rebuildGraph(s, s.m, true);


  const resultEl = document.getElementById('result');
  resultEl.innerHTML =
    `<div class="eva-row"><input type="text" id="eva-input" value="${expr}" title="Ops: + - * /  (integer)&#10;e=emb.dim  g=gaps  f=Frobenius  t=type  m=mult&#10;s=σ (elts below c)  r=reflected gaps&#10;Q=largest gen  A=max Apéry (f+m)&#10;a[i]=Apéry[i]  q[i]=generator[i]"><span id="eva-result">= ${exprVal ?? '—'}</span><span class="eva-spacer"></span><label>offset: <span id="sg-offset-val">0</span></label><input type="range" id="sg-offset" min="0" max="${s.m - 1}" value="0"></div>` +
    `<div id="sg-grid-container" class="table-wrap"></div>` +
    `<div id="classify" class="classify-row table-wrap">${js_classify_table(s)}</div>`;

  // Live expression evaluator
  document.getElementById('eva-input').addEventListener('input', e => {
    state_set_eva_expr(e.target.value);
    document.getElementById('eva-result').textContent = `= ${eval_expr(e.target.value, s) ?? '—'}`;
  });

  const slider = document.getElementById('sg-offset');
  const renderGrid = () => {
    document.getElementById('sg-grid-container').innerHTML = combined_table(s, parseInt(slider.value, 10), 0, state_get_show_kunz());
  };
  renderGrid();
  slider.addEventListener('input', () => {
    document.getElementById('sg-offset-val').textContent = slider.value;
    renderGrid();
  });

  document.getElementById('tilt-controls').innerHTML =
    `<div class="eva-row"><label>tilt: <span id="tilt-tilt-val">0</span></label><input type="range" id="tilt-tilt" min="${-s.m}" max="${s.m}" value="0"></div>`;
  const tiltInput = document.getElementById('tilt-tilt');
  const renderTiltGrid = () => {
    document.getElementById('tilt-grid-container').innerHTML = tilt_table(s, parseInt(tiltInput.value, 10));
  };
  renderTiltGrid();
  tiltInput.addEventListener('input', e => {
    document.getElementById('tilt-tilt-val').textContent = e.target.value;
    renderTiltGrid();
  });

  if (document.getElementById('tab-diagonals').classList.contains('active')) {
    renderDiagonals(s);
  }

  // Sortable classify-table: clicking a <th> sorts by that column.
  document.querySelectorAll('#classify .classify-table th').forEach((th, col) => {
    th.addEventListener('click', () => {
      const table = th.closest('table');
      const sortBody = table.querySelector('tbody');
      const asc = th.dataset.sort !== 'asc';
      table.querySelectorAll('th').forEach(h => delete h.dataset.sort);
      th.dataset.sort = asc ? 'asc' : 'desc';
      Array.from(sortBody.querySelectorAll('tr'))
        .sort((a, b) => {
          const av = a.cells[col].textContent.trim();
          const bv = b.cells[col].textContent.trim();
          const an = parseFloat(av), bn = parseFloat(bv);
          const order = (!isNaN(an) && !isNaN(bn)) ? an - bn : av.localeCompare(bv);
          return asc ? order : -order;
        })
        .forEach(r => sortBody.appendChild(r));
    });
  });

  // Help tab: dynamic description of the current semigroup
  {
    const sg = n => `<span class="sg-gen">${n}</span>`;
    const sf = n => `<span class="sg-frob">${n}</span>`;
    const sp = n => `<span class="sg-pf">${n}</span>`;
    const gensSpans = Array.from(s.gen_set).map(sg).join(', ');
    const pf = Array.from(s.pf);
    const pfStr = pf.length ? pf.map(sp).join(', ') : '—';
    document.getElementById('help-dynamic').innerHTML =
      `<p>The semigroup <strong>S&nbsp;=&nbsp;&#x27E8;${gensSpans}&#x27E9;</strong> has:</p>
      <ul>
        <li>Multiplicity <strong>m = ${sg(s.m)}</strong></li>
        <li>Frobenius number <strong>f = ${sf(s.f)}</strong></li>
        <li>Embedding dimension <strong>e = ${s.e}</strong> (= number of minimal generators)</li>
        <li>Number of gaps <strong>g = ${s.count_gap}</strong></li>
        <li>Conductor <strong>c = ${s.f + 1}</strong>, elements below conductor: ${s.count_set}</li>
        <li>Type <strong>t = ${s.type_t}</strong> — pseudo-Frobenius numbers: ${pfStr}</li>
        <li>${s.is_symmetric ? 'S is <strong>symmetric</strong> (t&nbsp;=&nbsp;1, g&nbsp;=&nbsp;(f+1)/2)' : 'S is <strong>not symmetric</strong>'}</li>
      </ul>`;
  }

  requestAnimationFrame(() => {
    const w = document.getElementById('current-prop-tbody').closest('table').offsetWidth;
    const histTable = document.querySelector('#tab-history .history-table');
    if (w > (parseFloat(histTable.style.minWidth) || 0)) { histTable.style.minWidth = `${w}px`; }
  });

  // Re-render 3D Kunz view if that tab is currently active.
  if (document.getElementById('tab-gapgraph').classList.contains('active')) { render3d(s, doToggle); }

  document.getElementById('add-pf-btn').style.display = (s.type_t > 1 && s.f > 0) ? '' : 'none';
  document.getElementById('add-blobs-btn').style.display = s.blob.length > 0 ? '' : 'none';
  document.getElementById('selfglue-btn').style.display = s.can_self_glue() ? '' : 'none';

  if (document.getElementById('tab-csv').classList.contains('active')) { buildCsv(); }
  if (document.getElementById('tab-latex').classList.contains('active')) { buildLatex(s); }
}

// Toggle a generator or gap: clicking a labelled number in the result panel.
function doToggle(val) {
  if (!currentS) { return; }
  const newS = currentS.toggle(val);
  const newGens = Array.from(newS.gen_set);
  if (newGens.length === 0 || newGens.join(',') === currentGenSet.join(',')) { return; }
  const sign = currentS.is_element(val) ? '-' : '+';
  const toggle = { sign, cls: js_node_class(currentS, val), n: val, from: state_current_idx() };
  const canonical = newGens.join(', ');
  gensInput.value = canonical;
  const idx = state_push(canonical);
  if (idx < 0) { return; } // no positive generators (shouldn't happen on toggle, but defensive)
  const sg = state_get(idx);
  if (!sg) { return; }
  render(sg, toggle, `${sign}${val}`);
  history.pushState({ gens: canonical, idx }, '', `?g=${encodeURIComponent(canonical)}`);
}

// Parse the generator input, push to Rust state, and render.
function compute() {
  if (guardBusy()) { return; }
  const raw = gensInput.value.trim();
  const errEl = document.getElementById('error');

  errEl.style.display = 'none';
  document.getElementById('result').innerHTML = '';

  if (!raw) { return; }

  // Normalise: sort numerically, then use canonical form throughout
  const canonical = parseGens(raw).sort((a, b) => a - b).join(', ');
  gensInput.value = canonical;

  setBusy(true);
  const label = _computeLabel;
  _computeLabel = '⏎'; // reset for next manual compute
  try {
    const idx = state_push(canonical);
    if (idx < 0) {
      showError('Need at least one positive generator.');
      return;
    }
    const sg = state_get(idx);
    if (!sg) { return; }
    render(sg, null, label);
    if (!navigating) { history.pushState({ gens: canonical, idx }, '', `?g=${encodeURIComponent(canonical)}`); }
  } catch (e) {
    showError(`Error: ${e.message ?? e}`);
  } finally {
    setBusy(false);
  }
}

// Generate 8 random integers in [10, 100].
function randNums() {
  return Array.from({ length: 8 }, () => Math.floor(Math.random() * 91) + 10);
}

// "RndKf": random generators + generators [k*m .. (k+1)*m] to push f near k*m.
function randWithMultiplier(k) {
  const nums = randNums();
  const peek = js_compute(nums.join(', ')); // peek without storing
  if (!peek) { return; }
  const m = peek.m;
  const extra = Array.from({ length: k * m + 1 }, (_, i) => k * m + i);
  gensInput.value = [...nums, ...extra].join(', ');
  compute();
}

function renderDiagonals(s) {
  document.getElementById('diagonals-container').innerHTML = js_diagonals_table(s);
}

// Activate the named tab and deactivate all others; trigger 3D render if needed.
function switchTab(name) {
  let activeBtn = null;
  document.querySelectorAll('.tab-btn').forEach(b => {
    const on = b.dataset.tab === name;
    b.classList.toggle('active', on);
    if (on) { activeBtn = b; }
  });
  document.querySelectorAll('.tab-content').forEach(c => c.classList.toggle('active', c.id === `tab-${name}`));
  // On mobile the tab bar is horizontally scrollable; keep the active tab in view.
  if (activeBtn) {
    activeBtn.scrollIntoView({ behavior: 'smooth', inline: 'nearest', block: 'nearest' });
  }
  if (name === 'gapgraph' && currentS) { render3d(currentS, doToggle); }
  if (name === 'csv') { buildCsv(); }
  if (name === 'latex' && currentS) { buildLatex(currentS); }
  if (name === 'diagonals' && currentS) { renderDiagonals(currentS); }
}
document.querySelectorAll('.tab-btn').forEach(b => b.addEventListener('click', () => switchTab(b.dataset.tab)));

// Wire a "Copy" button: copy getContent() to clipboard, flash "Copied!" for 1.5 s.
function setupCopyButton(btnId, getContent) {
  document.getElementById(btnId).addEventListener('click', e => {
    const btn = e.currentTarget;
    navigator.clipboard.writeText(getContent()).then(() => {
      btn.textContent = 'Copied!';
      setTimeout(() => { btn.textContent = 'Copy'; }, 1500);
    });
  });
}

setupCopyButton('gap-copy-btn', () => document.getElementById('history-gap').textContent);
setupCopyButton('csv-copy-btn', () => document.getElementById('csv-output').value);

await init();

document.getElementById('latex-source').addEventListener('input', e => renderLatexPreview(e.target.value));

document.getElementById('current-prop-thead').innerHTML = PROP_THEAD_TR;
document.querySelector('#tab-history .history-table thead').innerHTML = PROP_THEAD_TR;

setupGraphUpto(() => currentS);
setupShowGaps(() => currentS, () => Number(document.getElementById('graph-upto').value));
setupGraphToggle(val => doToggle(val));

// Apéry-cell / residue-sep hover: highlight Kunz matrix rows, columns, anti-diagonals.
// Residue-sep hover also highlights column (m-k) mod m as a secondary highlight.
// Delegates from #result (always in DOM) because #sg-grid-container is created by render().
{
  const resultEl = document.getElementById('result');
  let activeK = null;
  let activeSource = null; // 'apery' or 'sep'

  const clearHighlight = () => {
    if (activeK === null) { return; }
    activeK = null;
    activeSource = null;
    const grid = document.getElementById('sg-grid-container');
    grid?.querySelectorAll('.kunz-highlight, .kunz-highlight-2')
      .forEach(el => el.classList.remove('kunz-highlight', 'kunz-highlight-2'));
  };

  resultEl.addEventListener('mouseover', e => {
    const aperyTd = e.target.closest('.apery-row td[data-k]');
    const sepTh = !aperyTd ? e.target.closest('.residue-sep[data-k]') : null;
    const source = aperyTd ? 'apery' : (sepTh ? 'sep' : null);
    const k = aperyTd ? aperyTd.dataset.k : (sepTh ? sepTh.dataset.k : null);
    if (k === activeK && source === activeSource) { return; }
    clearHighlight();
    if (!k) { return; }
    activeK = k;
    activeSource = source;
    const grid = document.getElementById('sg-grid-container');
    if (!grid) { return; }
    if (source === 'apery' && state_get_show_kunz()) {
      grid.querySelectorAll(`td[data-kunz-i="${k}"], td[data-kunz-j="${k}"], td[data-kunz-sum="${k}"]`)
        .forEach(el => el.classList.add('kunz-highlight'));
    } else if (source === 'sep') {
      const m = currentS ? currentS.m : 0;
      const f = currentS ? currentS.f : 0;
      const mirror = ((f - Number(k)) % m + m) % m;
      grid.querySelectorAll(`td[data-kunz-j="${k}"], td[data-res="${k}"]`)
        .forEach(el => el.classList.add('kunz-highlight'));
      if (mirror !== Number(k)) {
        grid.querySelectorAll(`td[data-kunz-j="${mirror}"], td[data-res="${mirror}"]`)
          .forEach(el => el.classList.add('kunz-highlight-2'));
      }
    }
  });

  resultEl.addEventListener('mouseleave', clearHighlight);
}

document.getElementById('show-classification').addEventListener('change', function () {
  state_set_show_classification(this.checked);
  document.body.classList.toggle('hide-classification', !this.checked);
});

document.getElementById('show-kunz').addEventListener('change', function () {
  state_set_show_kunz(this.checked);
  const slider = document.getElementById('sg-offset');
  if (currentS && slider) {
    document.getElementById('sg-grid-container').innerHTML = combined_table(currentS, parseInt(slider.value, 10), 0, this.checked);
  }
});

// Toggle .open on .has-popup cells when the count is clicked (touch / click support).
document.addEventListener('click', e => {
  const popup = e.target.closest('.has-popup');
  const isSpan = e.target.closest('span[data-n], span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (popup && !isSpan) {
    // close all others, toggle this one
    document.querySelectorAll('.has-popup.open').forEach(el => { if (el !== popup) { el.classList.remove('open'); } });
    popup.classList.toggle('open');
  } else if (!popup) {
    document.querySelectorAll('.has-popup.open').forEach(el => el.classList.remove('open'));
  }
});

// Clicking a span in the shared shortprop row toggles without switching tabs.
document.getElementById('current-prop-tbody').addEventListener('click', e => {
  if (guardBusy()) { return; }
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) { return; }
  const row = e.target.closest('.history-row');
  if (!row) { return; }
  const rowIdx = parseInt(row.dataset.idx, 10);
  state_set_current_idx(rowIdx);
  const sg = state_get(rowIdx);
  if (!sg) { return; }
  currentS = sg;
  currentGenSet = Array.from(currentS.gen_set);
  doToggle(parseInt(span.textContent, 10));
});

// Clicking a history row re-renders that semigroup in the main tab.
document.getElementById('history-tbody').addEventListener('click', e => {
  const cell = e.target.closest('td');
  const row = e.target.closest('.history-row');
  if (!row || !cell) { return; }
  const rowIdx = parseInt(row.dataset.idx, 10);
  const s = state_get(rowIdx);
  if (!s) { return; }
  if (cell.cellIndex === 0) {
    const canonical = Array.from(s.gen_set).join(', ');
    gensInput.value = canonical;
    switchTab('s');
    const newIdx = state_push(canonical);
    if (newIdx < 0) { return; }
    const newSg = state_get(newIdx);
    if (!newSg) { return; }
    render(newSg, null, '⏎');
    history.pushState({ gens: canonical, idx: newIdx }, '', `?g=${encodeURIComponent(canonical)}`);
    return;
  }
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) { return; }
  currentS = s;
  state_set_current_idx(rowIdx);
  currentGenSet = Array.from(s.gen_set);
  doToggle(parseInt(span.textContent, 10));
});

// Browser back/forward: restore from WASM history index if available,
// otherwise recompute from generators (e.g. on page reload).
window.addEventListener('popstate', e => {
  if (e.state?.idx != null && e.state.idx < state_len()) {
    state_set_current_idx(e.state.idx);
    navigating = true;
    const sg = state_get(e.state.idx);
    if (sg) { render(sg); }
    navigating = false;
  } else if (e.state?.gens) {
    gensInput.value = e.state.gens;
    navigating = true;
    compute();
    navigating = false;
  }
});

// Tilt tab: click-to-toggle, stays on tilt tab.
document.getElementById('tilt-grid-container').addEventListener('click', e => {
  if (guardBusy()) { return; }
  const sgSpan = e.target.closest('.sg-grid span[data-n]');
  if (sgSpan) { doToggle(parseInt(sgSpan.dataset.n, 10)); }
});

// Delegate click-to-toggle from property spans and grid cells.
document.getElementById('result').addEventListener('click', e => {
  if (guardBusy()) { return; }
  const span = e.target.closest('span.sg-frob, span.sg-pf, span.sg-pf-blob, span.sg-gen, span[data-remove-gen]');
  if (span) { doToggle(parseInt(span.textContent, 10)); return; }
  const sgSpan = e.target.closest('.sg-grid span[data-n], .classify-table span[data-n]');
  if (sgSpan) { doToggle(parseInt(sgSpan.dataset.n, 10)); }
});

const guardedCompute = guarded(compute);

document.getElementById('random-btn').addEventListener('click', guarded(() => {
  gensInput.value = randNums().join(', ');
  _computeLabel = 'Rnd';
  compute();
}));

document.getElementById('random3f-btn').addEventListener('click', guarded(() => { _computeLabel = 'Rnd3m'; randWithMultiplier(3); }));
document.getElementById('random2f-btn').addEventListener('click', guarded(() => { _computeLabel = 'Rnd2m'; randWithMultiplier(2); }));

// "Symmetric": retry random samples until a symmetric semigroup is found.
document.getElementById('random-symmetric-btn').addEventListener('click', guarded(() => {
  for (let attempt = 0; attempt < 10000; attempt++) {
    const nums = randNums();
    if (js_compute(nums.join(', ')).is_symmetric) { // peek without storing
      gensInput.value = nums.join(', ');
      _computeLabel = 'Sym';
      compute();
      return;
    }
  }
  showError('Could not find a symmetric semigroup after 10000 attempts.');
}));

// "Prime": pick a random subset of 4–8 primes from the fixed list.
document.getElementById('random-primes-btn').addEventListener('click', guarded(() => {
  const count = Math.floor(Math.random() * 5) + 4; // 4..8
  const chosen = PRIMES_LIST.slice().sort(() => Math.random() - 0.5).slice(0, count).sort((a, b) => a - b);
  gensInput.value = chosen.join(', ');
  _computeLabel = 'P';
  compute();
}));

// "T(m,f)": generate semigroup <m, f+1, f+2, ..., f+m>; default f=2m when only m given.
document.getElementById('tmf-btn').addEventListener('click', guarded(() => {
  const nums = parseGens(gensInput.value);
  let m, f;
  if (nums.length === 0) { m = 2; f = 2 * m; }
  else if (nums.length === 1) { [m] = nums; f = 2 * m; }
  else { [m, f] = nums; }
  gensInput.value = [m, ...Array.from({ length: m }, (_, i) => f + 1 + i)].join(', ');
  _computeLabel = `T(${m},${f})`;
  compute();
}));

// "A(m,d,n)": generate arithmetic sequence m, m+d, m+2d, ..., m+nd.
document.getElementById('amdn-btn').addEventListener('click', guarded(() => {
  const nums = parseGens(gensInput.value);
  let m, d, n;
  if (nums.length === 0) { m = 3; d = 1; n = 3; }
  else if (nums.length === 1) { [m] = nums; d = 1; n = 3; }
  else if (nums.length === 2) { [m, d] = nums; n = 3; }
  else { [m, d, n] = nums; }
  gensInput.value = Array.from({ length: n + 1 }, (_, i) => m + i * d).join(', ');
  _computeLabel = `A(${m},${d},${n})`;
  compute();
}));

// "Rolf": p_n and all primes > p_n up to 5·p_n.
// If input contains "->", parse as "n->m" and compute Rolf for each index n..m.
document.getElementById('rolf-btn').addEventListener('click', guarded(() => {
  const raw = gensInput.value.trim();
  const arrow = raw.match(/^(\d+)\s*->\s*(\d+)$/);
  if (arrow) {
    const from = parseInt(arrow[1], 10);
    const to = parseInt(arrow[2], 10);
    const step = from <= to ? 1 : -1;
    setBusy(true);
    try {
      let lastIdx, lastCanonical;
      for (let i = from; ; i += step) {
        const primes = Array.from(js_rolf_primes(i));
        lastCanonical = primes.join(', ');
        lastIdx = state_push(lastCanonical);
        // Append history row without full render for intermediate entries
        if (i !== to) {
          const s = state_get(lastIdx);
          const histTbody = document.getElementById('history-tbody');
          if (lastIdx > 0 && lastIdx % 10 === 0) {
            histTbody.insertAdjacentHTML('beforeend', PROP_THEAD_TR);
          }
          histTbody.insertAdjacentHTML('beforeend', historyRow(s, lastIdx, `Rolf(${i})`, null, state_get_eva_expr(), eval_expr(state_get_eva_expr(), s), '—'));
        }
        if (i === to) { break; }
      }
      gensInput.value = lastCanonical;
      render(state_get(lastIdx), null, `Rolf(${to})`);
      history.pushState({ gens: lastCanonical, idx: lastIdx }, '', `?g=${encodeURIComponent(lastCanonical)}`);
    } catch (e) {
      showError(`Error: ${e.message ?? e}`);
    } finally {
      setBusy(false);
    }
  } else {
    const nums = parseGens(raw);
    const n = nums.length > 0 ? nums[0] : 3;
    const primes = Array.from(js_rolf_primes(n));
    gensInput.value = primes.join(', ');
    _computeLabel = `Rolf(${n})`;
    compute();
  }
}));

function wireGenSetBtn(id, method, label, beforeCompute) {
  document.getElementById(id).addEventListener('click', guarded(() => {
    if (!currentS) { return; }
    const gens = Array.from(currentS[method]());
    if (gens.length === 0) { return; }
    gensInput.value = gens.join(', ');
    _computeLabel = label;
    if (beforeCompute) { beforeCompute(); }
    compute();
  }));
}

wireGenSetBtn('half-btn',         's_over_2',           'S/2');
wireGenSetBtn('sym-partner-btn',  'symmetric_partner',  'S=SYM/2');
wireGenSetBtn('ks-btn',           'canonical_ideal',    'K(S)');
wireGenSetBtn('add-pf-btn',    'add_all_pf',         '+PF');
wireGenSetBtn('add-blobs-btn', 'add_reflected_gaps',  '+refl');
wireGenSetBtn('selfglue-btn',  'self_glue',           'glue', () => {
  state_set_show_kunz(false);
  document.getElementById('show-kunz').checked = false;
});

document.getElementById('compute-btn').addEventListener('click', guardedCompute);
document.getElementById('reset-btn').addEventListener('click', guarded(() => {
  gensInput.value = '';
}));
gensInput.addEventListener('keydown', e => { if (e.key === 'Enter') { guardedCompute(); } });

// On load: use URL param ?g=... if present, otherwise compute the default input.
const urlGens = new URLSearchParams(location.search).get('g');
if (urlGens) {
  gensInput.value = urlGens;
}
compute();
// Store the initial history index so back/forward can find it.
history.replaceState({ gens: gensInput.value, idx: state_current_idx() }, '', location.href);
