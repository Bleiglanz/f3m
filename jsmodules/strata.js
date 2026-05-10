import init, {
  js_strata_empty, js_strata_random, js_strata_table, js_strata_toggle,
  js_compute, combined_table,
} from '../pkg/semigroup_explorer.js';

await init();

const nInput = document.getElementById('strata-n');
const lmaxInput = document.getElementById('strata-lmax');
const mInput = document.getElementById('strata-m');
const out = document.getElementById('strata-out');
const sgOut = document.getElementById('strata-sg-out');
const errEl = document.getElementById('error');

let chain = '';
let n = readN();
let m = readM();

function readN() {
  const v = parseInt(nInput.value, 10);
  return Number.isFinite(v) && v > 0 ? v : 1;
}

function readLmax() {
  const v = parseInt(lmaxInput.value, 10);
  return Number.isFinite(v) && v >= 0 ? v : 0;
}

// Enforce m > N. If the user typed something smaller, silently clamp the
// input to N+1 so what they see is what's used.
function readM() {
  const v = parseInt(mInput.value, 10);
  const minM = readN() + 1;
  const clamped = Number.isFinite(v) && v >= minM ? v : minM;
  if (clamped !== v) { mInput.value = clamped; }
  return clamped;
}

function showError(msg) {
  errEl.textContent = msg;
  errEl.style.display = msg ? 'block' : 'none';
}

window.addEventListener('error', e => showError(`JS error: ${e.message}`));
window.addEventListener('unhandledrejection', e => showError(`Unhandled promise rejection: ${e.reason}`));

// Build generators for the side-by-side semigroup view:
// • m (the multiplicity itself)
// • every defined w_i = (min l with i ∈ M_l) * m + i
// • a tail of m consecutive integers starting at lmax+N+1, which guarantees
//   the resulting semigroup has finite complement.
//
// Returns null when no w_i is defined yet — caller falls back to ⟨2,3⟩ so
// the empty chain shows a simple proxy instead of a confusing generic one.
function buildGeneratorString(currentChain, lmax) {
  const rows = currentChain
    .split(';')
    .map(r => (r ? r.split(',').map(s => parseInt(s, 10)).filter(Number.isFinite) : []));
  const wValues = [];
  for (let i = 1; i <= n; i++) {
    for (let l = 0; l < rows.length; l++) {
      if (rows[l].includes(i)) {
        wValues.push(l * m + i);
        break;
      }
    }
  }
  if (wValues.length === 0) { return null; }
  const gens = [m, ...wValues];
  for (let k = 0; k < m; k++) {
    gens.push(lmax + n + 1 + k);
  }
  return gens.join(',');
}

function renderSemigroup() {
  const input = buildGeneratorString(chain, readLmax()) ?? '2,3';
  const sg = js_compute(input);
  if (!sg) {
    sgOut.innerHTML = '<em>(no valid semigroup for these inputs)</em>';
    return;
  }
  sgOut.innerHTML = combined_table(sg, 0, 0, false, false);
}

function render() {
  out.innerHTML = js_strata_table(chain, n, m);
  renderSemigroup();
}

function compute() {
  showError('');
  n = readN();
  m = readM();
  chain = js_strata_empty(readLmax());
  render();
}

function randomise() {
  showError('');
  n = readN();
  m = readM();
  chain = js_strata_random(n, readLmax());
  render();
}

document.getElementById('strata-compute').addEventListener('click', compute);
document.getElementById('strata-rnd1').addEventListener('click', randomise);
[nInput, lmaxInput, mInput].forEach(el =>
  el.addEventListener('keydown', e => { if (e.key === 'Enter') { compute(); } }));

// N and lmax changes resize the table → reset the chain and re-render.
nInput.addEventListener('change', compute);
lmaxInput.addEventListener('change', compute);
// Changing m alone re-renders without resetting the chain.
mInput.addEventListener('change', () => { m = readM(); render(); });

// Click-to-toggle: monotonicity is enforced server-side.
out.addEventListener('click', e => {
  const td = e.target.closest('td[data-l][data-v]');
  if (!td) { return; }
  const l = parseInt(td.dataset.l, 10);
  const v = parseInt(td.dataset.v, 10);
  if (l === 0) { return; }  // M_0 stays empty by definition
  chain = js_strata_toggle(chain, l, v);
  render();
});

// Hover highlighting:
// • Any cell or column header with data-v lights up columns i and N+1-i (TODO 77).
// • Hovering an empty data cell additionally lights up other empty cells whose
//   value equals val(hovered) + w_j for some defined w_j (TODO 79).
let hoveredCell = null;

function clearHighlights() {
  out.querySelectorAll('.strata-col-hl, .strata-gap-hl, .strata-self-hl')
    .forEach(el => el.classList.remove('strata-col-hl', 'strata-gap-hl', 'strata-self-hl'));
}

out.addEventListener('mouseover', e => {
  const cell = e.target.closest('[data-v]');
  if (!cell || cell === hoveredCell) { return; }
  clearHighlights();
  hoveredCell = cell;

  const v = parseInt(cell.dataset.v, 10);
  const mirror = n + 1 - v;
  out.querySelectorAll(`[data-v="${v}"], [data-v="${mirror}"]`)
    .forEach(el => el.classList.add('strata-col-hl'));

  // Gap + w_j highlighting only fires on empty data cells.
  if (cell.classList.contains('strata-out') && cell.dataset.val) {
    const hoveredVal = parseInt(cell.dataset.val, 10);
    cell.classList.add('strata-self-hl');
    const wValues = Array.from(out.querySelectorAll('tr.strata-w-val td[data-w]'))
      .map(td => parseInt(td.dataset.w, 10));
    for (const wj of wValues) {
      const target = hoveredVal + wj;
      out.querySelectorAll(`td.strata-out[data-val="${target}"]`)
        .forEach(el => el.classList.add('strata-gap-hl'));
    }
  }
});

out.addEventListener('mouseleave', () => { clearHighlights(); hoveredCell = null; });

// Initial render: empty chain at the default sizes.
compute();
