import init, {
  js_compute, combined_table, tilt_table, shortprop_tds, eval_expr,
  js_graph_edges_text, js_node_class, js_classify_table,
  state_push, state_get, state_current_idx, state_set_current_idx,
  state_get_eva_expr, state_set_eva_expr, state_gap_output, state_cmp,
  state_get_show_kunz, state_set_show_kunz,
  state_get_show_classification, state_set_show_classification,
} from '../pkg/f3m.js';
import { render3d } from './view3d.js';
import { rebuildGraph, setupGraphUpto, setupShowGaps } from './graph.js';

const PROP_THEAD_TR = '<tr><th>#</th><th>toggle</th><th>m</th><th>f</th><th>e</th><th>g</th><th style="white-space:nowrap">c&#8209;g</th><th>t</th><th>Sym</th><th>gen</th><th>PF</th><th>SPF</th><th>expr</th><th>value</th><th>&#8838;?</th></tr>';

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

document.getElementById('show-classification').addEventListener('change', function() {
  state_set_show_classification(this.checked);
  document.body.classList.toggle('hide-classification', !this.checked);
});

document.getElementById('show-kunz').addEventListener('change', function() {
  state_set_show_kunz(this.checked);
  const slider = document.getElementById('sg-offset');
  if (currentS && slider) {
    document.getElementById('sg-grid-container').innerHTML = combined_table(currentS, parseInt(slider.value), 0, this.checked);
  }
});

// Toggle .open on .has-popup cells when the count is clicked (touch / click support).
document.addEventListener('click', e => {
  const popup = e.target.closest('.has-popup');
  const isSpan = e.target.closest('span[data-n], span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (popup && !isSpan) {
    // close all others, toggle this one
    document.querySelectorAll('.has-popup.open').forEach(el => { if (el !== popup) el.classList.remove('open'); });
    popup.classList.toggle('open');
  } else if (!popup) {
    document.querySelectorAll('.has-popup.open').forEach(el => el.classList.remove('open'));
  }
});

// ── UI-only state (Rust owns the data) ────────────────────────────────────────
let currentGenSet = null; // gen array of the currently displayed semigroup
let currentS = null;      // JsSemigroup currently displayed (rendering cache)
let computing = false;    // true while a computation is running (guards re-entry)
let navigating = false;   // true when compute() is triggered by popstate (skip pushState)
const busyBanner = document.getElementById('busy-banner');

// Build a history table row for semigroup `s` at index `idx`.
const sIdx = n => `S<sub>${n}</sub>`;

function historyRow(s, idx, toggle, expr, value, cmp) {
  const valStr = value ?? '—';
  const toggleStr = toggle
    ? `${sIdx(toggle.from)}${toggle.sign}<span class="${toggle.cls}">${toggle.n}</span>`
    : '—';
  return `<tr class="history-row" data-idx="${idx}"><td>${sIdx(idx)}</td><td>${toggleStr}</td>${shortprop_tds(s)}<td class="left">${expr}</td><td>${valStr}</td><td>${cmp}</td></tr>`;
}

// Clicking a span in the shared shortprop row toggles without switching tabs.
document.getElementById('current-prop-tbody').addEventListener('click', e => {
  if (guardBusy()) return;
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) return;
  const row = e.target.closest('.history-row');
  if (!row) return;
  const rowIdx = parseInt(row.dataset.idx);
  state_set_current_idx(rowIdx);
  currentS = state_get(rowIdx);
  currentGenSet = Array.from(currentS.gen_set);
  doToggle(parseInt(span.textContent));
});

// Clicking a history row re-renders that semigroup in the main tab.
document.getElementById('history-tbody').addEventListener('click', e => {
  const cell = e.target.closest('td');
  const row = e.target.closest('.history-row');
  if (!row || !cell) return;
  const rowIdx = parseInt(row.dataset.idx);
  const s = state_get(rowIdx);
  if (cell.cellIndex === 0) {
    const canonical = Array.from(s.gen_set).join(', ');
    gensInput.value = canonical;
    switchTab('s');
    render(state_get(state_push(canonical)));
    return;
  }
  const span = e.target.closest('span.sg-gen, span.sg-frob, span.sg-pf, span.sg-pf-blob');
  if (!span) return;
  currentS = s;
  state_set_current_idx(rowIdx);
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

// Parse a generator string: split on any non-digit sequence, keep positive integers.
const parseGens = str => str.split(/\D+/).map(t => parseInt(t)).filter(n => !isNaN(n) && n > 0);

// Render a semigroup: update all UI tabs. Caller must call state_push first.
function render(s, toggle = null) {
  currentGenSet = Array.from(s.gen_set);
  currentS = s;
  const idx = state_current_idx();
  const expr = state_get_eva_expr();
  const exprVal = eval_expr(expr, s);
  const cmpSourceIdx = toggle ? toggle.from : (idx > 0 ? idx - 1 : null);
  const cmp = cmpSourceIdx != null
    ? `${sIdx(idx)}&nbsp;${state_cmp(idx, cmpSourceIdx)}&nbsp;${sIdx(cmpSourceIdx)}`
    : '—';
  const rowHtml = historyRow(s, idx, toggle, expr, exprVal, cmp);

  // History tab
  document.getElementById('history-tbody').insertAdjacentHTML('beforeend', rowHtml);
  document.getElementById('history-gap').textContent = state_gap_output();

  // Top property row + graph tab
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
    [`<input type="text" id="eva-input" value="${expr}" style="width:100%" title="Ops: + - * /  (integer)&#10;e=emb.dim  g=gaps  f=Frobenius  t=type  m=mult&#10;Q=largest gen  A=max Apéry (f+m)&#10;a[i]=Apéry[i]  q[i]=generator[i]">`,
     `<td class="value" id="eva-result">${exprVal ?? '—'}</td>`],
    ['# reflected&nbsp;gaps',  tipCell(blobs.length, blobs.join(', '))],
    ['Largest generator (ae)',      `<td class="value"><span class="sg-gen">${s.max_gen}</span></td>`],
    ['Structure / <span class="has-tip">c<sub>ij</sub><span class="tip">apery(i) + apery(j) − apery((i+j) mod m) / m</span></span>',
      `<td class="value sg-cell"><div class="sg-slider-row"><label>offset: <span id="sg-offset-val">0</span></label><input type="range" id="sg-offset" min="0" max="${s.m - 1}" value="0"></div><div id="sg-grid-container"></div></td>`],
    ['Classification (0…f+m)',      `<td class="value"><div id="classify">${js_classify_table(s)}</div></td>`, 'classify-row'],
  ];

  const tbody = rows.map(([label, td, cls]) => `<tr${cls ? ` class="${cls}"` : ''}><td class="label">${label}</td>${td}</tr>`).join('');

  const resultEl = document.getElementById('result');
  resultEl.innerHTML = `<table><tbody>${tbody}</tbody></table>`;

  // Live expression evaluator
  document.getElementById('eva-input').addEventListener('input', e => {
    state_set_eva_expr(e.target.value);
    document.getElementById('eva-result').textContent = eval_expr(e.target.value, s) ?? '—';
  });

  const slider = document.getElementById('sg-offset');
  const renderGrid = () => {
    document.getElementById('sg-grid-container').innerHTML = combined_table(s, parseInt(slider.value), 0, state_get_show_kunz());
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

  // Sortable classify-table: clicking a <th> sorts by that column.
  document.querySelectorAll('#classify .classify-table th').forEach((th, col) => {
    th.addEventListener('click', () => {
      const table = th.closest('table');
      const tbody = table.querySelector('tbody');
      const asc = th.dataset.sort !== 'asc';
      table.querySelectorAll('th').forEach(h => delete h.dataset.sort);
      th.dataset.sort = asc ? 'asc' : 'desc';
      Array.from(tbody.querySelectorAll('tr'))
        .sort((a, b) => {
          const av = a.cells[col].textContent.trim();
          const bv = b.cells[col].textContent.trim();
          const an = parseFloat(av), bn = parseFloat(bv);
          const cmp = (!isNaN(an) && !isNaN(bn)) ? an - bn : av.localeCompare(bv);
          return asc ? cmp : -cmp;
        })
        .forEach(r => tbody.appendChild(r));
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
    if (w > (parseFloat(histTable.style.minWidth) || 0)) histTable.style.minWidth = w + 'px';
  });

  // Trigger 3D view re-render if that tab is currently active
  if (document.getElementById('tab-gapgraph').classList.contains('active')) render3d(s);
}

// Parse the generator input, push to Rust state, and render.
function compute() {
  if (guardBusy()) return;
  const raw = gensInput.value.trim();
  const errEl = document.getElementById('error');

  errEl.style.display = 'none';
  document.getElementById('result').innerHTML = '';

  if (!raw) { return; }

  // Normalise: sort numerically, then use canonical form throughout
  const canonical = parseGens(raw).sort((a, b) => a - b).join(', ');
  gensInput.value = canonical;

  setBusy(true);
  try {
    render(state_get(state_push(canonical)));
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
  const toggle = { sign, cls: js_node_class(currentS, val), n: val, from: state_current_idx() };
  const canonical = newGens.join(', ');
  gensInput.value = canonical;
  render(state_get(state_push(canonical)), toggle);
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

// "RndKf": random generators + generators [k*m .. (k+1)*m] to push f near k*m.
function randWithMultiplier(k) {
  const nums = randNums();
  const m = js_compute(nums.join(', ')).m; // peek without storing
  const extra = Array.from({length: k * m + 1}, (_, i) => k * m + i);
  gensInput.value = [...nums, ...extra].join(', ');
  compute();
}
document.getElementById('random3f-btn').addEventListener('click', guarded(() => randWithMultiplier(3)));
document.getElementById('random2f-btn').addEventListener('click', guarded(() => randWithMultiplier(2)));

// "Symmetric": retry random samples until a symmetric semigroup is found.
document.getElementById('random-symmetric-btn').addEventListener('click', guarded(() => {
  for (let attempt = 0; attempt < 10000; attempt++) {
    const nums = randNums();
    if (js_compute(nums.join(', ')).is_symmetric) { // peek without storing
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
  const chosen = PRIMES_LIST.slice().sort(() => Math.random() - 0.5).slice(0, count).sort((a, b) => a - b);
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
  gensInput.value = [m, ...Array.from({length: m - 1}, (_, i) => k * m + 1 + i)].join(', ');
  compute();
}));

// "A(m,d,n)": generate arithmetic sequence m, m+d, m+2d, ..., m+nd.
document.getElementById('amdn-btn').addEventListener('click', guarded(() => {
  const nums = parseGens(gensInput.value);
  let m, d, n;
  if (nums.length === 0)      { m = 3; d = 1; n = 3; }
  else if (nums.length === 1) { m = nums[0]; d = 1; n = 3; }
  else if (nums.length === 2) { m = nums[0]; d = nums[1]; n = 3; }
  else                        { m = nums[0]; d = nums[1]; n = nums[2]; }
  gensInput.value = Array.from({length: n + 1}, (_, i) => m + i * d).join(', ');
  compute();
}));

document.getElementById('compute-btn').addEventListener('click', guardedCompute);
document.getElementById('reset-btn').addEventListener('click', guarded(() => {
  gensInput.value = '6, 9, 20';
  compute();
}));
gensInput.addEventListener('keydown', e => { if (e.key === 'Enter') guardedCompute(); });

// On load: use URL param ?g=... if present, otherwise compute the default input.
const urlGens = new URLSearchParams(location.search).get('g');
if (urlGens) {
  gensInput.value = urlGens;
  history.replaceState({gens: urlGens}, '', location.href);
}
compute();
