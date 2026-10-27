// Shared structure-grid CSS-class predicates and visibility rule, used by both
// the Graph view (graph.js) and the 3D view (view3d.js). The structure-grid
// counterpart is the body.hide-* rules in style.css.
import { state_get_show_gaps, state_get_show_s, state_get_show_refls } from '../pkg/semigroup_explorer.js';

// True if `cls` is a reflected-gap class (a gap n where f−n is also a gap).
function isReflClass(cls) {
  return cls === 'sg-blob' || cls === 'sg-pf-blob';
}

// True if `cls` is a non-reflected gap class.
function isPlainGapClass(cls) {
  return cls === 'sg-out' || cls === 'sg-frob' || cls === 'sg-pf';
}

// Current Gaps / S / refls display toggles from the WASM page state.
export function visibilityToggles() {
  return { gaps: state_get_show_gaps(), s: state_get_show_s(), refls: state_get_show_refls() };
}

// Whether a node/mesh of class `cls` is visible under `toggles`. Reflected gaps
// are governed solely by "refls" — they show when refls is on even if the
// broader "Gaps" toggle is off. Every other class is an element of S.
export function classVisible(cls, toggles) {
  if (isReflClass(cls)) { return toggles.refls; }
  if (isPlainGapClass(cls)) { return toggles.gaps; }
  return toggles.s;
}
