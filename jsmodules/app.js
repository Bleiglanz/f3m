import init, { js_compute, combined_table, shortprop_tds, eval_expr, js_gap_block, js_graph_edges_text, gap_header, gap_footer, js_node_class, js_classify_table, js_cmp_semigroups } from '../pkg/f3m.js';
import { render3d } from './view3d.js';
import { rebuildGraph, setupGraphUpto, setupShowGaps } from './graph.js';

const PROP_THEAD_TR = '<tr><th>#</th><th>toggle</th><th>m</th><th>f</th><th>e</th><th>g</th><th>c-g</th><th>t</th><th>Sym</th><th>gen</th><th>PF</th><th>SPF</th><th>expr</th><th>value</th><th>⊆?</th></tr>';

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

document.querySelector('#tab-graph .history-table thead').innerHTML = PROP_THEAD_TR;
document.querySelector('#tab-history .history-table thead').innerHTML = PROP_THEAD_TR;

setupGraphUpto(() => currentS);
setupShowGaps(() => currentS, () => Number(document.getElementById('graph-upto').value));

// ── App state ─────────────────────────────────────────────────────────────────
let currentGenSet = null; // generator array of the most recently rendered semigroup
let currentS = null;      // JsSemigroup WASM object currently displayed
let evaExpr = 'f+1';      // expression shown in the evaluator input
let computing = false;    // true while a computation is running (guards re-entry)
const busyBanner = document.getElementById('busy-banner');
const historyList = []; // all JsSemigroup objects computed this session
let gapBlocks = '';    // accumulated js_gap_block output, without header/footer

// Build a history table row for semigroup `s` at index `idx`.
function historyRow(s, idx, toggle, expr, value, cmp) {
  const valStr = value ?? '—';
  const toggleStr = toggle
    ? `${toggle.sign}<span class="${toggle.cls}">${toggle.n}</span>`
    : '—';
  return `<tr class="history-row" data-idx="${idx}"><td>${idx}</td><td>${toggleStr}</td>${shortprop_tds(s)}<td class="left">${expr}</td><td>${valStr}</td><td>${cmp}</td></tr>`;
}

// Clicking a span in the graph tab's shortprop row toggles without switching tabs.
document.getElementById('graph-prop-tbody').addEventListener('click', e => {
  if (guardBusy()) return;
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf');
  if (!span) return;
  const row = e.target.closest('.history-row');
  if (!row) return;
  const s = historyList[parseInt(row.dataset.idx)];
  currentS = s;
  currentGenSet = Array.from(s.gen_set);
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
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf');
  if (!span) return;
  currentS = s;
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

// Render a semigroup: update all tabs, history, GAP output, and the main property table.
function render(s, toggle = null) {
  currentGenSet = Array.from(s.gen_set);
  currentS = s;
  historyList.push(s);
  const idx = historyList.length - 1;
  const exprVal = eval_expr(evaExpr, s);
  const cmp = idx > 0 ? js_cmp_semigroups(s, historyList[idx - 1]) : '—';
  const rowHtml = historyRow(s, idx, toggle, evaExpr, exprVal, cmp);

  // History tab
  document.getElementById('history-tbody').insertAdjacentHTML('beforeend', rowHtml);
  gapBlocks += js_gap_block(s, idx + 1);
  document.getElementById('history-gap').textContent = gap_header() + gapBlocks + gap_footer();

  // Graph tab
  document.getElementById('graph-prop-tbody').innerHTML = rowHtml;
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
      `<td class="value sg-cell"><div class="sg-slider-row"><label>offset: <span id="sg-offset-val">0</span></label><input type="range" id="sg-offset" min="0" max="${s.m - 1}" value="0">${s.m <= 15 ? `&nbsp;&nbsp;<label>tilt: <span id="sg-tilt-val">0</span></label><input type="range" id="sg-tilt" min="${-s.m}" max="${s.m}" value="0">` : ''}</div><div id="sg-grid-container"></div></td>`],
    ['Classification (0…f+m)',      `<td class="value"><div id="classify">${js_classify_table(s)}</div></td>`],
  ];

  const tbody = rows.map(([label, td]) => `<tr><td class="label">${label}</td>${td}</tr>`).join('');

  const resultEl = document.getElementById('result');
  resultEl.innerHTML = `
    <table class="history-table">
      <thead>${PROP_THEAD_TR}</thead>
      <tbody>${rowHtml}</tbody>
    </table>
    <table>
      <tbody>${tbody}</tbody>
    </table>`;

  // Live expression evaluator
  document.getElementById('eva-input').addEventListener('input', e => {
    evaExpr = e.target.value;
    const result = eval_expr(evaExpr, s);
    document.getElementById('eva-result').textContent = result ?? '—';
  });

  // Structure/Kunz grid with offset and tilt sliders
  const slider = document.getElementById('sg-offset');
  const tiltSlider = document.getElementById('sg-tilt');
  const renderGrid = () => {
    const tilt = tiltSlider ? parseInt(tiltSlider.value) : 0;
    document.getElementById('sg-grid-container').innerHTML = combined_table(s, parseInt(slider.value), tilt);
  };
  renderGrid();
  slider.addEventListener('input', () => {
    document.getElementById('sg-offset-val').textContent = slider.value;
    renderGrid();
  });
  if (tiltSlider) {
    tiltSlider.addEventListener('input', () => {
      document.getElementById('sg-tilt-val').textContent = tiltSlider.value;
      renderGrid();
    });
  }

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
  const sorted = raw.split(',').map(t => parseInt(t.trim())).filter(n => !isNaN(n)).sort((a, b) => a - b);
  gensInput.value = sorted.join(', ');

  setBusy(true);
  try {
    render(js_compute(raw));
  } catch (e) {
    showError('Error: ' + (e.message ?? e));
  } finally {
    setBusy(false);
  }
}

// Toggle a generator or gap: clicking a labelled number in the result panel.
function doToggle(val) {
  if (!currentS) return;
  const newS = currentS.toggle(val);
  const newGens = Array.from(newS.gen_set);
  if (newGens.length === 0 || newGens.join(',') === currentGenSet.join(',')) return;
  const sign = currentS.is_element(val) ? '-' : '+';
  const toggle = { sign, cls: js_node_class(currentS, val), n: val };
  gensInput.value = newGens.join(', ');
  render(newS, toggle);
}

// Delegate click-to-toggle from property spans and grid cells.
document.getElementById('result').addEventListener('click', e => {
  if (guardBusy()) return;
  const span = e.target.closest('span.sg-frob, span.sg-pf, span.sg-gen, span[data-remove-gen]');
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

function isPrime(n) {
  if (n < 2) return false;
  for (let i = 2; i * i <= n; i++) if (n % i === 0) return false;
  return true;
}

// "RandomPrimes": consecutive primes starting from a random base ≥ 3.
const guardedCompute = guarded(compute);
document.getElementById('random-primes-btn').addEventListener('click', guarded(() => {
  const x = Math.floor(Math.random() * 48) + 3;
  const primes = Array.from({length: 5 * x + 1}, (_, i) => x + i).filter(isPrime);
  gensInput.value = primes.join(', ');
  compute();
}));

document.getElementById('compute-btn').addEventListener('click', guardedCompute);
gensInput.addEventListener('keydown', e => { if (e.key === 'Enter') guardedCompute(); });

compute();
