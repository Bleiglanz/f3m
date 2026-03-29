// Three.js renderer state — kept module-level so render3d() can clean up on re-render.
let _3dAnimId = null;    // requestAnimationFrame handle
let _3dRenderer = null;  // WebGLRenderer instance
let _3dCamera = null;    // current PerspectiveCamera (for saving state on teardown)
let _3dControls = null;  // current OrbitControls
let _3dGapMeshes = [];  // cube meshes classified as gaps (for show/hide)
let _3dSMeshes = [];    // cube meshes classified as semigroup elements (for show/hide)
// Persisted camera state so toggles/recomputes don't reset the viewpoint.
let _3dCameraState = null; // { position, target, zoom }

// CSS colors from style.css, mapped to structure-table classes.
// bg = cube face color, fg = text color (used for Apéry tiles).
const COLORS = {
  'sg-gen':     { bg: '#7ab3e8', fg: '#111' },     // minimal generator
  'sg-apery':   { bg: '#1a5fb4', fg: '#fff' },     // Apéry element (non-generator)
  'sg-frob':    { bg: '#ffd6d3', fg: '#7a1008' },  // Frobenius number
  'sg-pf':      { bg: '#e8d5ff', fg: '#4a1a80' },  // pseudo-Frobenius
  'sg-pf-blob': { bg: '#d2f0d2', fg: '#4a1a80' },  // PF ∩ reflected gap
  'sg-blob':    { bg: '#d2f0d2', fg: '#0f4a0f' },  // reflected gap
  'sg-in':      { bg: '#222222', fg: '#fff' },      // element of S
  'sg-out':     { bg: '#f7f8fa', fg: '#999' },      // gap
};

// Human-readable labels for each CSS class.
const CLS_LABEL = {
  'sg-gen':     'generator',
  'sg-apery':   'Apéry',
  'sg-frob':    'Frobenius',
  'sg-pf':      'pseudo-Frob',
  'sg-pf-blob': 'pseudo-Frob+refl',
  'sg-blob':    'reflected gap',
  'sg-in':      '∈ S',
  'sg-out':     'gap',
};

// Classify a number n into its structure-table CSS class (same logic as Rust get_cls).
function classify(n, f, m, apery, genSet, pfSet, blobSet) {
  if (n === f)                           { return 'sg-frob'; }
  if (genSet.has(n))                     { return 'sg-gen'; }
  if (n === apery[n % m])                { return 'sg-apery'; }
  if (n >= apery[n % m]) {
    return 'sg-in';
  }
  if (pfSet.has(n) && blobSet.has(n))    { return 'sg-pf-blob'; }
  if (pfSet.has(n))                      { return 'sg-pf'; }
  if (blobSet.has(n))                    { return 'sg-blob'; }
  return 'sg-out';
}

// True if the class represents a gap (hidden by "show gaps" unchecked).
function isGapClass(cls) {
  return cls === 'sg-out' || cls === 'sg-frob' || cls === 'sg-pf' || cls === 'sg-pf-blob' || cls === 'sg-blob';
}

// Create a flat 1×1 tile with class-colored background and number label,
// lying in the z=0 plane with its lower-left corner at (x, y, 0).
function aperyTile(text, x, y, cls) {
  const c = COLORS[cls] || COLORS['sg-apery'];
  const canvas = document.createElement('canvas');
  canvas.width = 128;
  canvas.height = 128;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = c.bg;
  ctx.fillRect(0, 0, 128, 128);
  ctx.strokeStyle = c.fg;
  ctx.lineWidth = 3;
  ctx.strokeRect(1, 1, 126, 126);
  ctx.font = 'bold 56px sans-serif';
  ctx.fillStyle = c.fg;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 64, 64);
  const texture = new THREE.CanvasTexture(canvas);
  const geo = new THREE.PlaneGeometry(1, 1);
  const mat = new THREE.MeshBasicMaterial({ map: texture, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.position.set(x + 0.5, y + 0.5, 0);
  return mesh;
}

// Toggle 3D cube visibility to match the show_gaps / show_s checkboxes.
export function update3dVisibility() {
  const showGaps = document.getElementById('graph-show-gaps').checked;
  const showS = document.getElementById('graph-show-s').checked;
  for (const m of _3dGapMeshes) { m.visible = showGaps; }
  for (const m of _3dSMeshes) { m.visible = showS; }
}

// Create a text-only sprite for axis labels.
function textSprite(text, color, size) {
  const canvas = document.createElement('canvas');
  canvas.width = 256;
  canvas.height = 64;
  const ctx = canvas.getContext('2d');
  ctx.font = 'bold 40px sans-serif';
  ctx.fillStyle = color;
  ctx.textAlign = 'center';
  ctx.fillText(text, 128, 46);
  const texture = new THREE.CanvasTexture(canvas);
  const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture }));
  sprite.scale.set(size * 2, size * 0.5, 1);
  return sprite;
}

// (Re-)render the 3D Kunz-coordinate view for any numerical semigroup.
//
// Axes: x = residue class i (0..m-1), y = Kunz coordinate k_i = (a_i - i)/m, z = multiples of m.
// z = 0 plane: Apéry tiles at (i, k_i), coloured and labelled by value.
// z < 0: gap cubes (value = a_i + z·m > 0 are gaps by Apéry minimality).
// z > 0: sporadic S-element cubes (value = a_i + z·m ≤ f).
// Colours match the CSS classes in style.css.
export function render3d(s, onToggle) {
  const container = document.getElementById('sg-3d-container');
  // Save camera state before teardown so the viewpoint persists across redraws.
  if (_3dCamera && _3dControls) {
    _3dCameraState = {
      position: _3dCamera.position.clone(),
      target: _3dControls.target.clone(),
    };
  }
  container.innerHTML = '';
  if (_3dAnimId) { cancelAnimationFrame(_3dAnimId); _3dAnimId = null; }
  if (_3dRenderer) { _3dRenderer.dispose(); _3dRenderer = null; }
  _3dCamera = null;
  _3dControls = null;

  const gens = Array.from(s.gen_set);
  const m = gens[0];
  const {f} = s;
  const apery = Array.from(s.apery_set);
  const genSet = new Set(gens);
  const pfSet = new Set(Array.from(s.pf));
  const blobSet = new Set(Array.from(s.blob));

  _3dGapMeshes = [];
  _3dSMeshes = [];

  // For each Apéry element, place at Kunz coordinates (i, k_i) and build z-column.
  const aperyPoints = []; // { x, y, val, cls }
  const cubes = [];       // { x, y, z, val, cls }

  for (let i = 0; i < m; i++) {
    const a = apery[i];
    const ki = (a - i) / m; // Kunz coordinate (always a non-negative integer)
    const aCls = classify(a, f, m, apery, genSet, pfSet, blobSet);
    aperyPoints.push({ x: i, y: ki, val: a, cls: aCls });

    // z < 0: gaps (a - k·m > 0 are all gaps by Apéry minimality)
    for (let z = -1; ; z--) {
      const val = a + z * m;
      if (val <= 0) { break; }
      cubes.push({ x: i, y: ki, z, val, cls: classify(val, f, m, apery, genSet, pfSet, blobSet) });
    }

    // z > 0: sporadic elements of S (below conductor f+1)
    for (let z = 1; ; z++) {
      const val = a + z * m;
      if (val > f) { break; }
      cubes.push({ x: i, y: ki, z, val, cls: classify(val, f, m, apery, genSet, pfSet, blobSet) });
    }
  }

  const W = container.offsetWidth || 800;
  const H = 500;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xd0d0d0);

  // Determine scene extent from data
  let xMax = m - 1, yMax = 1, zMin = 0, zMax = 0;
  for (const p of aperyPoints) {
    if (p.y > yMax) { yMax = p.y; }
  }
  for (const c of cubes) {
    if (c.z < zMin) { zMin = c.z; }
    if (c.z > zMax) { zMax = c.z; }
  }
  const extent = Math.max(xMax, yMax, Math.abs(zMin), zMax, 4);

  const camera = new THREE.PerspectiveCamera(50, W / H, 0.1, extent * 40);
  camera.up.set(0, 0, 1);
  camera.position.set(extent * 1.5, -extent * 1.5, extent * 1.2);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(W, H);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.appendChild(renderer.domElement);
  _3dRenderer = renderer;

  // ── Grid in the z=0 plane ─────────────────────────────────────────────────
  const gridW = xMax + 2;
  const gridH = yMax + 2;
  const gridSize = Math.max(gridW, gridH);
  const gridHelper = new THREE.GridHelper(gridSize, gridSize, 0xbbbbbb, 0xdddddd);
  gridHelper.rotation.x = Math.PI / 2;
  gridHelper.position.set(gridSize / 2, gridSize / 2, 0);
  scene.add(gridHelper);

  // ── Axes ─────────────────────────────────────────────────────────────────
  const axisLenX = xMax + 3;
  const axisLenY = yMax + 3;
  const axisMat = new THREE.LineMaterial({ color: 0x111111, linewidth: 2 });
  axisMat.resolution.set(W, H);

  const xGeo = new THREE.LineGeometry();
  xGeo.setPositions([0, 0, 0, axisLenX, 0, 0]);
  scene.add(new THREE.Line2(xGeo, axisMat));

  const yGeo = new THREE.LineGeometry();
  yGeo.setPositions([0, 0, 0, 0, axisLenY, 0]);
  scene.add(new THREE.Line2(yGeo, axisMat));

  const zAxisMat = new THREE.LineMaterial({ color: 0x0000ff, linewidth: 2 });
  zAxisMat.resolution.set(W, H);
  const zGeo = new THREE.LineGeometry();
  zGeo.setPositions([0, 0, zMin - 1, 0, 0, zMax + 1]);
  scene.add(new THREE.Line2(zGeo, zAxisMat));

  // Arrowheads
  const arrowLen = extent * 0.06;
  const arrowRad = arrowLen * 0.4;
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(1, 0, 0), new THREE.Vector3(axisLenX, 0, 0), arrowLen, 0x111111, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, axisLenY, 0), arrowLen, 0x111111, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, zMax + 1), arrowLen, 0x0000ff, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 0, -1), new THREE.Vector3(0, 0, zMin - 1), arrowLen, 0x0000ff, arrowLen, arrowRad));

  // Axis labels
  const labelSize = extent * 0.12;
  const xLabel = textSprite('residue i', '#111', labelSize);
  xLabel.position.set(axisLenX + extent * 0.15, 0, 0);
  scene.add(xLabel);
  const yLabel = textSprite('Kunz (a\u1d62\u2212i)/m', '#111', labelSize);
  yLabel.position.set(0, axisLenY + extent * 0.15, 0);
  scene.add(yLabel);
  const zLabel = textSprite(`\u00d7m (m=${m})`, '#00c', labelSize);
  zLabel.position.set(0, 0, zMax + 1 + extent * 0.12);
  scene.add(zLabel);

  // ── Apéry tiles (colored by class, labelled) in the z=0 plane ───────────
  const showGaps = document.getElementById('graph-show-gaps').checked;
  const showS = document.getElementById('graph-show-s').checked;
  const clickable = []; // all meshes that can be hovered/clicked

  for (const p of aperyPoints) {
    const tile = aperyTile(String(p.val), p.x, p.y, p.cls);
    tile.userData = { val: p.val, cls: p.cls };
    const gap = isGapClass(p.cls);
    tile.visible = gap ? showGaps : showS;
    scene.add(tile);
    clickable.push(tile);
    (gap ? _3dGapMeshes : _3dSMeshes).push(tile);
  }

  // ── Cubes colored by structure-table class ────────────────────────────────
  const cubeSize = 0.45;
  const boxGeo = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
  const matCache = {};
  function matFor(cls) {
    if (!matCache[cls]) {
      const c = COLORS[cls] || COLORS['sg-out'];
      matCache[cls] = new THREE.MeshBasicMaterial({
        color: new THREE.Color(c.bg), transparent: true, opacity: 0.85,
      });
    }
    return matCache[cls];
  }

  for (const c of cubes) {
    const mesh = new THREE.Mesh(boxGeo, matFor(c.cls));
    mesh.position.set(c.x + 0.5, c.y + 0.5, c.z);
    mesh.userData = { val: c.val, cls: c.cls };
    const gap = isGapClass(c.cls);
    mesh.visible = gap ? showGaps : showS;
    scene.add(mesh);
    clickable.push(mesh);
    (gap ? _3dGapMeshes : _3dSMeshes).push(mesh);
  }

  // ── Raycaster: hover highlight + tooltip + click-to-toggle ───────────────
  const raycaster = new THREE.Raycaster();
  const mouse = new THREE.Vector2();
  let hoveredMesh = null;
  let hoveredOriginalMat = null;
  const highlightMat = new THREE.MeshBasicMaterial({ color: 0xff8800, transparent: true, opacity: 0.95 });

  const tooltip = document.createElement('div');
  tooltip.style.cssText = 'position:absolute;pointer-events:none;background:#333;color:#fff;padding:2px 6px;border-radius:3px;font:bold 13px monospace;display:none;z-index:10';
  container.style.position = 'relative';
  container.appendChild(tooltip);

  // Raycast from a mouse/pointer event and return the first visible clickable mesh, or null.
  function hitTest(event) {
    const rect = renderer.domElement.getBoundingClientRect();
    mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    raycaster.setFromCamera(mouse, camera);
    const hits = raycaster.intersectObjects(clickable);
    return hits.length > 0 && hits[0].object.visible ? hits[0].object : null;
  }

  function clearHover() {
    if (hoveredMesh) {
      hoveredMesh.material = hoveredOriginalMat;
      hoveredMesh = null;
      hoveredOriginalMat = null;
    }
    tooltip.style.display = 'none';
    renderer.domElement.style.cursor = '';
  }

  renderer.domElement.addEventListener('mousemove', event => {
    const hit = hitTest(event);
    const rect = renderer.domElement.getBoundingClientRect();
    const tx = `${event.clientX - rect.left + 12}px`;
    const ty = `${event.clientY - rect.top - 20}px`;

    if (hit === hoveredMesh) {
      if (hit) { tooltip.style.left = tx; tooltip.style.top = ty; }
      return;
    }
    clearHover();
    if (hit) {
      hoveredMesh = hit;
      hoveredOriginalMat = hit.material;
      hit.material = highlightMat;
      tooltip.textContent = `${hit.userData.val} ${CLS_LABEL[hit.userData.cls] || ''}`;
      tooltip.style.display = 'block';
      tooltip.style.left = tx;
      tooltip.style.top = ty;
      renderer.domElement.style.cursor = 'pointer';
    }
  });

  renderer.domElement.addEventListener('mouseleave', clearHover);

  // Click-to-toggle: only fire if pointer barely moved (distinguishes from orbit drag).
  const DRAG_THRESHOLD_SQ = 16; // ~4px
  let pointerDownPos = null;
  renderer.domElement.addEventListener('pointerdown', event => {
    pointerDownPos = { x: event.clientX, y: event.clientY };
  });
  renderer.domElement.addEventListener('pointerup', event => {
    if (!pointerDownPos) { return; }
    const dx = event.clientX - pointerDownPos.x;
    const dy = event.clientY - pointerDownPos.y;
    pointerDownPos = null;
    if (dx * dx + dy * dy > DRAG_THRESHOLD_SQ) { return; }
    const hit = hitTest(event);
    if (hit && hit.userData.val != null && onToggle) { onToggle(hit.userData.val); }
  });

  // ── Controls & animation loop ────────────────────────────────────────────
  const controls = new THREE.OrbitControls(camera, renderer.domElement);
  if (_3dCameraState) {
    camera.position.copy(_3dCameraState.position);
    controls.target.copy(_3dCameraState.target);
  } else {
    const center = new THREE.Vector3(xMax / 2, yMax / 2, 0);
    controls.target.copy(center);
    camera.lookAt(center);
  }
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.update();
  _3dCamera = camera;
  _3dControls = controls;

  function animate() {
    _3dAnimId = requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();
}
