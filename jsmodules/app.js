import init, { js_compute, combined_table, tilt_table, shortprop_tds, eval_expr, js_gap_block, js_graph_edges_text, gap_header, gap_footer, js_node_class, js_classify_table, js_cmp_semigroups } from '../pkg/f3m.js';
import { render3d } from './view3d.js';
import { rebuildGraph, setupGraphUpto, setupShowGaps } from './graph.js';

const PROP_THEAD_TR = '<tr><th>#</th><th>toggle</th><th>m</th><th>f</th><th>e</th><th>g</th><th>c-g</th><th>t</th><th>Sym</th><th>gen</th><th>PF</th><th>expr</th><th>value</th><th>⊆?</th></tr>';

// Display an error message in the error banner.
function showError(msg) {
  const errEl = document.getElementById('error');
  errEl.textContent = msg;
  errEl.style.display = 'block';
}

window.addEventListener('error', e => {
  if (e.message?.includes('ResizeObserver')) return;
  showError('JS error: ' + e.message);
});
window.addEventListener('unhandledrejection', e => showError('Unhandled promise rejection: ' + e.reason));

// Activate the named tab and deactivate all others; trigger 3D render if needed.
function switchTab(name) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.toggle('active', b.dataset.tab === name));
  document.querySelectorAll('.tab-content').forEach(c => c.classList.toggle('active', c.id === 'tab-' + name));
  if (name === 'gapgraph' && currentS) render3d(currentS);
}
document.querySelectorAll('.tab-btn').forEach(b => b.addEventListener('click', () => switchTab(b.dataset.tab)));

// Copy the GAP script from the history panel to the clipboard.
document.getElementById('gap-copy-btn').addEventListener('click', e => {
  const btn = e.currentTarget;
  navigator.clipboard.writeText(document.getElementById('history-gap').textContent).then(() => {
    btn.textContent = 'Copied!';
    setTimeout(() => btn.textContent = 'Copy', 1500);
  });
});

await init();

document.getElementById('current-prop-thead').innerHTML = PROP_THEAD_TR;
document.querySelector('#tab-history .history-table thead').innerHTML = PROP_THEAD_TR;

setupGraphUpto(() => currentS);
setupShowGaps(() => currentS, () => Number(document.getElementById('graph-upto').value));

// ── App state ─────────────────────────────────────────────────────────────────
let currentGenSet = null; // generator array of the most recently rendered semigroup
let currentS = null;      // JsSemigroup WASM object currently displayed
let currentIdx = -1;      // index of currentS in historyList
let evaExpr = 'f+1';      // expression shown in the evaluator input
let computing = false;    // true while a computation is running (guards re-entry)
let navigating = false;   // true when compute() is triggered by popstate (skip pushState)
const busyBanner = document.getElementById('busy-banner');
const historyList = []; // all JsSemigroup objects computed this session
let gapBlocks = '';    // accumulated js_gap_block output, without header/footer

// Build a history table row for semigroup `s` at index `idx`.
function historyRow(s, idx, toggle, expr, value, cmp) {
  const valStr = value ?? '—';
  const toggleStr = toggle
    ? `${toggle.sign}#${toggle.from}<span class="${toggle.cls}">${toggle.n}</span>`
    : '—';
  return `<tr class="history-row" data-idx="${idx}"><td>${idx}</td><td>${toggleStr}</td>${shortprop_tds(s)}<td class="left">${expr}</td><td>${valStr}</td><td>${cmp}</td></tr>`;
}

// Clicking a span in the shared shortprop row toggles without switching tabs.
document.getElementById('current-prop-tbody').addEventListener('click', e => {
  if (guardBusy()) return;
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) return;
  const row = e.target.closest('.history-row');
  if (!row) return;
  currentIdx = parseInt(row.dataset.idx);
  currentS = historyList[currentIdx];
  currentGenSet = Array.from(currentS.gen_set);
  doToggle(parseInt(span.textContent));
});

// Clicking a history row re-renders that semigroup in the main tab.
document.getElementById('history-tbody').addEventListener('click', e => {
  const cell = e.target.closest('td');
  const row = e.target.closest('.history-row');
  if (!row || !cell) return;
  const s = historyList[parseInt(row.dataset.idx)];
  if (cell.cellIndex === 0) {
    gensInput.value = Array.from(s.gen_set).join(', ');
    switchTab('s');
    render(s);
    return;
  }
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) return;
  currentS = s;
  currentIdx = parseInt(row.dataset.idx);
  currentGenSet = Array.from(s.gen_set);
  doToggle(parseInt(span.textContent));
});

// Show/hide the "Computing…" banner.
function setBusy(b) {
  computing = b;
  busyBanner.style.display = b ? 'block' : 'none';
}

// Flash the banner if a new action is attempted while computing; return true if busy.
function guardBusy() {
  if (!computing) return false;
  busyBanner.classList.add('busy-flash');
  setTimeout(() => busyBanner.classList.remove('busy-flash'), 300);
  return true;
}

// Wrap a callback so it only runs when the app is not busy.
const guarded = fn => () => { if (!guardBusy()) fn(); };

const gensInput = document.getElementById('gens');

// Parse a comma-separated generator string into an array of positive integers.
const parseGens = str => str.split(',').map(t => parseInt(t.trim())).filter(n => !isNaN(n));

// Render a semigroup: update all tabs, history, GAP output, and the main property table.
function render(s, toggle = null) {
  currentGenSet = Array.from(s.gen_set);
  currentS = s;
  historyList.push(s);
  currentIdx = historyList.length - 1;
  const idx = currentIdx;
  const exprVal = eval_expr(evaExpr, s);
  const cmpSourceIdx = toggle ? toggle.from : (idx > 0 ? idx - 1 : null);
  const cmp = cmpSourceIdx != null
    ? `#${idx}&nbsp;${js_cmp_semigroups(s, historyList[cmpSourceIdx])}&nbsp;#${cmpSourceIdx}`
    : '—';
  const rowHtml = historyRow(s, idx, toggle, evaExpr, exprVal, cmp);

  // History tab
  document.getElementById('history-tbody').insertAdjacentHTML('beforeend', rowHtml);
  gapBlocks += js_gap_block(s, idx + 1);
  document.getElementById('history-gap').textContent = gap_header() + gapBlocks + gap_footer();

  // Graph tab
  document.getElementById('current-prop-tbody').innerHTML = rowHtml;
  const graphUpto = document.getElementById('graph-upto');
  graphUpto.min = s.m;
  graphUpto.max = s.f + s.m;
  graphUpto.value = s.m;
  document.getElementById('graph-upto-val').textContent = s.m;
  document.getElementById('graph-edges-text').value = js_graph_edges_text(s, s.m);
  rebuildGraph(s, s.m);

  // Cache WASM Vec getters — each call copies memory from WASM
  const blobs = Array.from(s.blob);

  // Helper: value cell with a hover tooltip.
  function tipCell(value, tipText) {
    return `<td class="value has-tip">${value}<span class="tip">${tipText}</span></td>`;
  }

  const rows = [
    ['Wilf&nbsp;sporadic/(f+1)',                   `<td class="value">${s.wilf.toFixed(4)} &gt;= ${(1/s.e).toFixed(4)}</td>`],
    ['Embedding&nbsp;dimension&nbsp;(e)', `<td class="value">${s.e}</td>`],
    [`<input type="text" id="eva-input" value="${evaExpr}" style="width:100%" title="Ops: + - * /  (integer)&#10;e=emb.dim  g=gaps  f=Frobenius  t=type  m=mult&#10;Q=largest gen  A=max Apéry (f+m)&#10;a[i]=Apéry[i]  q[i]=generator[i]">`,
     `<td class="value" id="eva-result">${exprVal ?? '—'}</td>`],
    ['# reflected&nbsp;gaps',  tipCell(blobs.length, blobs.join(', '))],
    ['Largest generator (ae)',      `<td class="value"><span class="sg-gen">${s.max_gen}</span></td>`],
    ['Structure / <span class="has-tip">c<sub>ij</sub><span class="tip">apery(i) + apery(j) − apery((i+j) mod m) / m</span></span>',
      `<td class="value sg-cell"><div class="sg-slider-row"><label>offset: <span id="sg-offset-val">0</span></label><input type="range" id="sg-offset" min="0" max="${s.m - 1}" value="0"></div><div id="sg-grid-container"></div></td>`],
    ['Classification (0…f+m)',      `<td class="value"><div id="classify">${js_classify_table(s)}</div></td>`],
  ];

  const tbody = rows.map(([label, td]) => `<tr><td class="label">${label}</td>${td}</tr>`).join('');

  const resultEl = document.getElementById('result');
  resultEl.innerHTML = `<table><tbody>${tbody}</tbody></table>`;

  // Live expression evaluator
  document.getElementById('eva-input').addEventListener('input', e => {
    evaExpr = e.target.value;
    const result = eval_expr(evaExpr, s);
    document.getElementById('eva-result').textContent = result ?? '—';
  });

  const slider = document.getElementById('sg-offset');
  const renderGrid = () => {
    document.getElementById('sg-grid-container').innerHTML = combined_table(s, parseInt(slider.value), 0);
  };
  renderGrid();
  slider.addEventListener('input', () => {
    document.getElementById('sg-offset-val').textContent = slider.value;
    renderGrid();
  });

  document.getElementById('tilt-controls').innerHTML =
    `<div class="sg-slider-row"><label>tilt: <span id="tilt-tilt-val">0</span></label><input type="range" id="tilt-tilt" min="${-s.m}" max="${s.m}" value="0"></div>`;
  const tiltInput = document.getElementById('tilt-tilt');
  const renderTiltGrid = () => {
    document.getElementById('tilt-grid-container').innerHTML = tilt_table(s, parseInt(tiltInput.value));
  };
  renderTiltGrid();
  tiltInput.addEventListener('input', e => {
    document.getElementById('tilt-tilt-val').textContent = e.target.value;
    renderTiltGrid();
  });

  requestAnimationFrame(() => {
    const w = document.getElementById('current-prop-tbody').closest('table').offsetWidth;
    const histTable = document.querySelector('#tab-history .history-table');
    if (w > (parseFloat(histTable.style.minWidth) || 0)) histTable.style.minWidth = w + 'px';
  });

  // Trigger 3D view re-render if that tab is currently active
  if (document.getElementById('tab-gapgraph').classList.contains('active')) render3d(s);
}

// Parse the generator input, sort it, and compute + render the semigroup.
function compute() {
  if (guardBusy()) return;
  const raw = gensInput.value.trim();
  const errEl = document.getElementById('error');

  errEl.style.display = 'none';
  document.getElementById('result').innerHTML = '';

  if (!raw) { return; }

  // Normalise display: sort numerically
  const sorted = parseGens(raw).sort((a, b) => a - b);
  const canonical = sorted.join(', ');
  gensInput.value = canonical;

  setBusy(true);
  try {
    render(js_compute(raw));
    if (!navigating) history.pushState({gens: canonical}, '', '?g=' + encodeURIComponent(canonical));
  } catch (e) {
    showError('Error: ' + (e.message ?? e));
  } finally {
    setBusy(false);
  }
}

// Browser back/forward: restore the generator set from history state.
window.addEventListener('popstate', e => {
  if (e.state?.gens) {
    gensInput.value = e.state.gens;
    navigating = true;
    compute();
    navigating = false;
  }
});

// Toggle a generator or gap: clicking a labelled number in the result panel.
function doToggle(val) {
  if (!currentS) return;
  const newS = currentS.toggle(val);
  const newGens = Array.from(newS.gen_set);
  if (newGens.length === 0 || newGens.join(',') === currentGenSet.join(',')) return;
  const sign = currentS.is_element(val) ? '-' : '+';
  const from = currentIdx;
  const toggle = { sign, cls: js_node_class(currentS, val), n: val, from };
  gensInput.value = newGens.join(', ');
  render(newS, toggle);
}

// Tilt tab: click-to-toggle, stays on tilt tab.
document.getElementById('tilt-grid-container').addEventListener('click', e => {
  if (guardBusy()) return;
  const sgSpan = e.target.closest('.sg-grid span[data-n]');
  if (sgSpan) doToggle(parseInt(sgSpan.dataset.n));
});

// Delegate click-to-toggle from property spans and grid cells.
document.getElementById('result').addEventListener('click', e => {
  if (guardBusy()) return;
  const span = e.target.closest('span.sg-frob, span.sg-pf, span.sg-pf-blob, span.sg-gen, span[data-remove-gen]');
  if (span) { doToggle(parseInt(span.textContent)); return; }
  const sgSpan = e.target.closest('.sg-grid span[data-n], .classify-table span[data-n]');
  if (sgSpan) doToggle(parseInt(sgSpan.dataset.n));
});

// Generate 8 random integers in [10, 100].
function randNums() {
  return Array.from({length: 8}, () => Math.floor(Math.random() * 91) + 10);
}

document.getElementById('random-btn').addEventListener('click', guarded(() => {
  gensInput.value = randNums().join(', ');
  compute();
}));

// "Random3f": random generators + enough multiples of m to force a large Frobenius number.
document.getElementById('random3f-btn').addEventListener('click', guarded(() => {
  const nums = randNums();
  const tempS = js_compute(nums.join(', '));
  const m = tempS.m;
  const extra = Array.from({length: 3 * m + 1}, (_, i) => 3 * m + i);
  const combined = [...nums, ...extra];
  gensInput.value = combined.join(', ');
  compute();
}));

// "Symmetric": retry random samples until a symmetric semigroup is found.
document.getElementById('random-symmetric-btn').addEventListener('click', guarded(() => {
  for (let attempt = 0; attempt < 10000; attempt++) {
    const nums = randNums();
    const tempS = js_compute(nums.join(', '));
    if (tempS.is_symmetric) {
      gensInput.value = nums.join(', ');
      compute();
      return;
    }
  }
  showError('Could not find a symmetric semigroup after 10000 attempts.');
}));

const PRIMES_LIST = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97];

// "Prime": pick a random subset of 4–8 primes from the fixed list.
const guardedCompute = guarded(compute);
document.getElementById('random-primes-btn').addEventListener('click', guarded(() => {
  const count = Math.floor(Math.random() * 5) + 4; // 4..8
  const shuffled = PRIMES_LIST.slice().sort(() => Math.random() - 0.5);
  const chosen = shuffled.slice(0, count).sort((a, b) => a - b);
  gensInput.value = chosen.join(', ');
  compute();
}));

// "H(m,k)": generate semigroup <m, km+1, km+2, ..., km+m-1>.
document.getElementById('hmk-btn').addEventListener('click', guarded(() => {
  const nums = parseGens(gensInput.value);
  let m, k;
  if (nums.length === 0)      { m = 2; k = 3; }
  else if (nums.length === 1) { m = nums[0]; k = 1; }
  else                        { m = nums[0]; k = nums[1]; }
  const gens = [m, ...Array.from({length: m - 1}, (_, i) => k * m + 1 + i)];
  gensInput.value = gens.join(', ');
  compute();
}));

document.getElementById('compute-btn').addEventListener('click', guardedCompute);
gensInput.addEventListener('keydown', e => { if (e.key === 'Enter') guardedCompute(); });

// On load: use URL param ?g=... if present, otherwise compute the default input.
const urlGens = new URLSearchParams(location.search).get('g');
if (urlGens) {
  gensInput.value = urlGens;
  history.replaceState({gens: urlGens}, '', location.href);
}
compute();
