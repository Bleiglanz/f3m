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
  out.innerHTML = js_strata_table(chain, n);
}

function compute() {
  showError('');
  n = readN();
  chain = js_strata_empty(readLmax());
  render();
}

function randomise() {
  showError('');
  n = readN();
  chain = js_strata_random(n, readLmax());
  render();
}

document.getElementById('strata-compute').addEventListener('click', compute);
document.getElementById('strata-rnd1').addEventListener('click', randomise);
nInput.addEventListener('keydown', e => { if (e.key === 'Enter') { compute(); } });
lmaxInput.addEventListener('keydown', e => { if (e.key === 'Enter') { compute(); } });

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

// Initial render: empty chain at the default sizes.
compute();
