import init, {
  js_strata_empty, js_strata_random, js_strata_table, js_strata_toggle,
} from '../pkg/semigroup_explorer.js';

await init();

const nInput = document.getElementById('strata-n');
const lmaxInput = document.getElementById('strata-lmax');
const out = document.getElementById('strata-out');
const errEl = document.getElementById('error');

let chain = '';
let n = readN();
// m is fixed at N + 1 (the semigroup convention where columns 1..N enumerate
// the non-zero residues mod m). It's not a user input.
let m = n + 1;

function readN() {
  const v = parseInt(nInput.value, 10);
  return Number.isFinite(v) && v > 0 ? v : 1;
}

function readLmax() {
  const v = parseInt(lmaxInput.value, 10);
  return Number.isFinite(v) && v >= 0 ? v : 0;
}

function showError(msg) {
  errEl.textContent = msg;
  errEl.style.display = msg ? 'block' : 'none';
}

window.addEventListener('error', e => showError(`JS error: ${e.message}`));
window.addEventListener('unhandledrejection', e => showError(`Unhandled promise rejection: ${e.reason}`));

function render() {
  out.innerHTML = js_strata_table(chain, n, m);
}

function compute() {
  showError('');
  n = readN();
  m = n + 1;
  chain = js_strata_empty(readLmax());
  render();
}

function randomise() {
  showError('');
  n = readN();
  m = n + 1;
  chain = js_strata_random(n, readLmax());
  render();
}

document.getElementById('strata-compute').addEventListener('click', compute);
document.getElementById('strata-rnd1').addEventListener('click', randomise);
[nInput, lmaxInput].forEach(el =>
  el.addEventListener('keydown', e => { if (e.key === 'Enter') { compute(); } }));

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
