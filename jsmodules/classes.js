// Shared structure-grid CSS-class predicates and visibility-checkbox readers,
// used by both the Graph view (graph.js) and the 3D view (view3d.js) so the
// "what counts as a reflected gap" rule lives in exactly one place.

// True if `cls` is a reflected-gap class (a gap n where f−n is also a gap).
// This is the single source of truth keyed on by the "refls" toggle; the
// matching CSS lives in style.css under `body.hide-refls`.
export function isReflClass(cls) {
  return cls === 'sg-blob' || cls === 'sg-pf-blob';
}

// Read the "refls" checkbox; defaults to visible if the element is absent.
export function reflsChecked() {
  const el = document.getElementById('show-refls');
  return el ? el.checked : true;
}
