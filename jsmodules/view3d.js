// Three.js renderer state — kept module-level so render3d() can clean up on re-render.
let _3dAnimId = null;   // requestAnimationFrame handle
let _3dRenderer = null; // WebGLRenderer instance

// (Re-)render the Wilf 3D visualisation for semigroup `s`.
//
// Coordinate axes: x = genus (g), y = conductor (f+1), z = height.
// The scene shows:
//   • Bold black x/y axes with arrowheads, thin blue z axis.
//   • A unit step-1 grid in the z=0 plane.
//   • The Wilf line y = (1 + 1/e) * x in red (boundary of Wilf's conjecture region).
//   • A black unit cube placed at grid point (g, f+1) representing the semigroup.
export function render3d(s) {
  const f = s.f;
  const e = s.e;
  const limit = 2 * f; // extent of axes and grid

  const container = document.getElementById('sg-3d-container');
  container.innerHTML = '';
  // Cancel previous animation loop and dispose GPU resources before re-creating.
  if (_3dAnimId)   { cancelAnimationFrame(_3dAnimId); _3dAnimId = null; }
  if (_3dRenderer) { _3dRenderer.dispose(); _3dRenderer = null; }

  const W = container.offsetWidth || 800;
  const H = 500;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xfafafa);

  // Camera starts top-down in the middle of the grid at height 2.5f.
  const camera = new THREE.PerspectiveCamera(50, W / H, 0.1, limit * 20);
  camera.up.set(0, 1, 0);
  camera.position.set(limit / 2, limit / 2, 2.5 * f);
  camera.lookAt(limit / 2, limit / 2, 0);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(W, H);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.appendChild(renderer.domElement);
  _3dRenderer = renderer;

  // ── Axes ─────────────────────────────────────────────────────────────────
  // x and y: bold black Line2 (supports linewidth > 1); z: thin blue standard line.
  const axisMatXY = new THREE.LineMaterial({ color: 0x111111, linewidth: 3 });
  axisMatXY.resolution.set(W, H);
  const xGeo = new THREE.LineGeometry(); xGeo.setPositions([0,0,0, limit,0,0]);
  scene.add(new THREE.Line2(xGeo, axisMatXY));
  const yGeo = new THREE.LineGeometry(); yGeo.setPositions([0,0,0, 0,limit,0]);
  scene.add(new THREE.Line2(yGeo, axisMatXY));
  const zGeo = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0,0,0), new THREE.Vector3(0,0,limit)]);
  scene.add(new THREE.Line(zGeo, new THREE.LineBasicMaterial({ color: 0x0000ff })));
  // Arrowheads at the positive ends of x and y.
  const arrowLen = limit * 0.08;
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(1,0,0), new THREE.Vector3(limit,0,0), arrowLen, 0x111111, arrowLen, arrowLen*0.4));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0,1,0), new THREE.Vector3(0,limit,0), arrowLen, 0x111111, arrowLen, arrowLen*0.4));

  // ── Grid ─────────────────────────────────────────────────────────────────
  // Step-1 grid in the z=0 plane, centred on (limit/2, limit/2).
  const grid = new THREE.GridHelper(limit, limit, 0xbbbbbb, 0xdddddd);
  grid.rotation.x = Math.PI / 2;
  grid.position.set(limit / 2, limit / 2, 0);
  scene.add(grid);

  // ── Wilf line ────────────────────────────────────────────────────────────
  // y = (1 + 1/e) * x separates semigroups satisfying Wilf's conjecture.
  const slope = 1 + 1 / e;
  const pos = [];
  for (let xi = 0; xi <= limit; xi++) {
    const yi = slope * xi;
    if (yi > limit) break;
    pos.push(xi, yi, 0);
  }
  const xEnd = limit / slope <= limit ? limit / slope : limit;
  pos.push(xEnd, slope * xEnd, 0);
  const lineGeo = new THREE.LineGeometry();
  lineGeo.setPositions(pos);
  const lineMat = new THREE.LineMaterial({ color: 0xdd0000, linewidth: 3 });
  lineMat.resolution.set(W, H);
  scene.add(new THREE.Line2(lineGeo, lineMat));

  // ── Semigroup cube ───────────────────────────────────────────────────────
  // Black unit cube centred on the integer grid point (g, f+1),
  // sitting on the z=0 plane (lower-left corner at (g-0.5, f+0.5, 0)).
  const dotGeo = new THREE.BoxGeometry(1, 1, 1);
  const dotMat = new THREE.MeshBasicMaterial({ color: 0x111111 });
  const dot = new THREE.Mesh(dotGeo, dotMat);
  dot.position.set(s.count_gap, s.f + 1, 0.5);
  scene.add(dot);

  // ── Controls & animation loop ────────────────────────────────────────────
  const controls = new THREE.OrbitControls(camera, renderer.domElement);
  controls.target.set(limit / 2, limit / 2, 0);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.update();

  function animate() {
    _3dAnimId = requestAnimationFrame(animate);
    controls.update(); // required for damping
    renderer.render(scene, camera);
  }
  animate();
}
