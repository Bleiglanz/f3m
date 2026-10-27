import { js_node_class, js_graph_node_ids, js_graph_edge_pairs, js_graph_edges_text,
         state_set_show_gaps, state_set_show_s, state_set_show_refls } from '../pkg/semigroup_explorer.js';
import { update3dVisibility } from './view3d.js';
import { classVisible, visibilityToggles } from './classes.js';

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
const graphNetwork = new vis.Network(
  graphVisEl,
  { nodes: graphNodes, edges: graphEdges },
  { edges: { arrows: 'to' } }
);

// Keep the graph canvas square whenever the wrapper is resized.
new ResizeObserver(entries => {
  const w = Math.round(entries[0].contentRect.width);
  graphVisEl.style.height = `${w}px`;
}).observe(document.getElementById('graph-resize-wrapper'));

// Rebuild the graph for semigroup `s` up to value `upto`, respecting visibility toggles.
// Edges whose endpoints are hidden are also suppressed.
export function rebuildGraph(s, upto, andFit = false) {
  const toggles = visibilityToggles();
  const nodeIds = js_graph_node_ids(s, upto);
  const edgePairs = js_graph_edge_pairs(s, upto);
  const visibleIds = new Set(Array.from(nodeIds).filter(n => classVisible(js_node_class(s, n), toggles)));
  graphNodes.clear();
  graphEdges.clear();
  graphNodes.add([...visibleIds].map(n => visNode(s, n)));
  for (let i = 0; i < edgePairs.length; i += 2) {
    const from = edgePairs[i], to = edgePairs[i + 1];
    if (visibleIds.has(from) && visibleIds.has(to))
      {graphEdges.add({ from, to });}
  }
  if (andFit) { graphNetwork.once('stabilized', () => graphNetwork.fit()); }
}

// Wire node clicks to toggle the clicked value.
export function setupGraphToggle(onToggle) {
  graphNetwork.on('click', params => {
    if (params.nodes.length === 1) { onToggle(params.nodes[0]); }
  });
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

// Wire the "Gaps", "S", and "refls" visibility checkboxes; all share the same
// rebuild callback and also refresh the 3D view.
export function setupVisibilityToggles(getCurrentS, getUpto) {
  const toggles = [
    ['graph-show-gaps', 'hide-gaps', state_set_show_gaps],
    ['graph-show-s', 'hide-s', state_set_show_s],
    ['show-refls', 'hide-refls', state_set_show_refls],
  ];
  for (const [id, hideClass, setState] of toggles) {
    document.getElementById(id).addEventListener('change', function() {
      setState(this.checked);
      document.body.classList.toggle(hideClass, !this.checked);
      const s = getCurrentS();
      if (s) {rebuildGraph(s, getUpto());}
      update3dVisibility();
    });
  }
}
