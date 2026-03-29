import { js_node_class, js_graph_node_ids, js_graph_edge_pairs, js_graph_edges_text,
         state_get_show_gaps, state_set_show_gaps, state_get_show_s, state_set_show_s } from '../pkg/f3m.js';
import { update3dVisibility } from './view3d.js';

// CSS class → vis-network background/border colors.
// Mirrors the color legend in style.css and CLAUDE.md.
const CLS_COLOR = {
  'sg-gen':   { background: '#7ab3e8', border: '#4a83b8' }, // minimal generator (blue)
  'sg-apery': { background: '#1a5fb4', border: '#0f3d7a' }, // Apéry element (dark blue)
  'sg-frob':  { background: '#ffcccc', border: '#cc8888' }, // Frobenius number (pink)
  'sg-pf':    { background: '#2d6a2d', border: '#1a3d1a' }, // pseudo-Frobenius (green)
  'sg-blob':  { background: '#b8e6b8', border: '#78b678' }, // reflected gap (light green)
  'sg-in':    { background: '#111', border: '#444', font: '#fff' }, // element of S (black, white label)
  'sg-out':   { background: '#f9f9f9', border: '#ccc' }, // gap (light grey)
};

// Build a vis-network color descriptor for a node class, with orange highlight border.
function nodeColor(cls) {
  const c = CLS_COLOR[cls] ?? { background: '#f9f9f9', border: '#ccc' };
  return { background: c.background, border: c.border, highlight: { background: c.background, border: '#ff6600' } };
}

// Build a vis-network node descriptor for number `n` in semigroup `s`.
function visNode(s, n) {
  const cls = s ? js_node_class(s, n) : 'sg-out';
  const c = CLS_COLOR[cls];
  const node = { id: n, label: String(n), color: nodeColor(cls) };
  if (c?.font) { node.font = { color: c.font }; } // white text on dark backgrounds
  return node;
}

const graphVisEl = document.getElementById('graph-vis');

// Initialise the vis-network Hasse diagram (directed edges = covering relations).
const graphNodes = new vis.DataSet([]);
const graphEdges = new vis.DataSet([]);
/* eslint-disable-next-line no-new */
new vis.Network(
  graphVisEl,
  { nodes: graphNodes, edges: graphEdges },
  { edges: { arrows: 'to' } }
);

// Keep the graph canvas square whenever the wrapper is resized.
new ResizeObserver(entries => {
  const w = Math.round(entries[0].contentRect.width);
  graphVisEl.style.height = `${w}px`;
}).observe(document.getElementById('graph-resize-wrapper'));

// CSS classes that belong to gaps (not elements of S).
const GAP_CLASSES = new Set(['sg-out', 'sg-blob', 'sg-pf', 'sg-frob']);
// CSS classes that belong to elements of S.
const S_CLASSES = new Set(['sg-in', 'sg-gen', 'sg-apery', 'sg-large']);

// Return true if node `n` should be shown given current toggle state.
function isVisible(s, n, showGaps, showS) {
  const cls = js_node_class(s, n);
  if (GAP_CLASSES.has(cls)) {return showGaps;}
  if (S_CLASSES.has(cls)) {return showS;}
  return true;
}

// Rebuild the graph for semigroup `s` up to value `upto`, respecting visibility toggles.
// Edges whose endpoints are hidden are also suppressed.
export function rebuildGraph(s, upto) {
  const showGaps = state_get_show_gaps();
  const showS = state_get_show_s();
  const nodeIds = js_graph_node_ids(s, upto);
  const edgePairs = js_graph_edge_pairs(s, upto);
  const visibleIds = new Set(Array.from(nodeIds).filter(n => isVisible(s, n, showGaps, showS)));
  graphNodes.clear();
  graphEdges.clear();
  graphNodes.add([...visibleIds].map(n => visNode(s, n)));
  for (let i = 0; i < edgePairs.length; i += 2) {
    const from = edgePairs[i], to = edgePairs[i + 1];
    if (visibleIds.has(from) && visibleIds.has(to))
      {graphEdges.add({ from, to });}
  }
}

// Wire the "Upto" slider: redraws the graph and updates the edge-text textarea.
export function setupGraphUpto(getCurrentS) {
  document.getElementById('graph-upto').addEventListener('input', function() {
    const upto = Number(this.value);
    document.getElementById('graph-upto-val').textContent = upto;
    const s = getCurrentS();
    if (s) {
      document.getElementById('graph-edges-text').value = js_graph_edges_text(s, upto);
      rebuildGraph(s, upto);
    }
  });
}

// Wire the "Show gaps" and "Show S" checkboxes; both share the same rebuild callback.
export function setupShowGaps(getCurrentS, getUpto) {
  const rebuild = () => { const s = getCurrentS(); if (s) {rebuildGraph(s, getUpto());} };
  document.getElementById('graph-show-gaps').addEventListener('change', function() {
    state_set_show_gaps(this.checked);
    document.body.classList.toggle('hide-gaps', !this.checked);
    rebuild();
    update3dVisibility();
  });
  document.getElementById('graph-show-s').addEventListener('change', function() {
    state_set_show_s(this.checked);
    document.body.classList.toggle('hide-s', !this.checked);
    rebuild();
    update3dVisibility();
  });
}
