// Three.js renderer state — kept module-level so render3d() can clean up on re-render.
let _3dAnimId = null;    // requestAnimationFrame handle
let _3dRenderer = null;  // WebGLRenderer instance
let _3dCamera = null;    // current PerspectiveCamera (for saving state on teardown)
let _3dControls = null;  // current OrbitControls
let _3dGapMeshes = [];  // cube meshes classified as gaps (for show/hide)
let _3dSMeshes = [];    // cube meshes classified as semigroup elements (for show/hide)

// ── Flatten animation state ──────────────────────────────────────��──────────
// States: 'kunz' (3D), 'to-flat', 'flat', 'to-kunz'
let _3dFlatState = 'kunz';
let _3dFlatT = 0;           // progress 0→1
const FLAT_DURATION = 1.2;  // seconds

// ── Descent/Ascent animation state ───────────────────────────────────────────
// Module-level handles so animate3dDescent/Ascent can find meshes built inside
// render3d() and queue a one-shot animation onto the existing rAF loop.
let _3dClickable = [];
let _3dCurrentM = 1;
let _3dPendingAnim = null;       // { type, t, duration, … }
const STEP_DURATION = 0.7;       // seconds — single descent/ascent animation

// Persisted camera state so toggles/recomputes don't reset the viewpoint.
let _3dCameraState = null; // { position, target, zoom }
const _3dHoverLineMat = new THREE.LineBasicMaterial({ color: 0xff8888, linewidth: 2, depthTest: false });

// CSS colours are read from style.css via the hidden #sg-color-probe element
// in index.html; this keeps the 3D view's cube/tile colours in sync with the
// rest of the UI instead of duplicating them as JS hex literals here.
//
// A small fallback table is used if the probe is missing (e.g. unit tests, or
// when the script runs before the probe is attached).
const FALLBACK_COLORS = {
  'sg-gen':     { bg: '#7ab3e8', fg: '#111' },
  'sg-apery':   { bg: '#1a5fb4', fg: '#fff' },
  'sg-frob':    { bg: '#ffd6d3', fg: '#7a1008' },
  'sg-pf':      { bg: '#e8d5ff', fg: '#4a1a80' },
  'sg-pf-blob': { bg: '#d2f0d2', fg: '#4a1a80' },
  'sg-blob':    { bg: '#d2f0d2', fg: '#0f4a0f' },
  'sg-in':      { bg: '#222222', fg: '#fff' },
  'sg-out':     { bg: '#f7f8fa', fg: '#999' },
};

// Theme-independent overrides for gap-class cubes in the 3D view. The 2D
// palette desaturates these in dark mode (so they read well as small cells
// against a dark page surface), but the 3D scene background is fixed light
// grey — desaturated dark colors blur into it and look uniformly black.
const VIEW3D_OVERRIDES = {
  'sg-out':     { bg: '#ffffff', fg: '#000' },  // plain gap → white
  'sg-frob':    { bg: '#dc2626', fg: '#fff' },  // Frobenius → red
  'sg-pf':      { bg: '#8a2be2', fg: '#fff' },  // pseudo-Frob → blue-violet
  'sg-pf-blob': { bg: '#ba55d3', fg: '#fff' },  // PF ∩ reflected gap → orchid
  'sg-blob':    { bg: '#84cc16', fg: '#000' },  // reflected gap → lime
};

// Lazily computed once on first call; recomputing per-frame would be wasteful.
let _colorCache = null;
function COLORS_FOR(cls) {
  if (VIEW3D_OVERRIDES[cls]) { return VIEW3D_OVERRIDES[cls]; }
  if (_colorCache === null) {
    _colorCache = readColorsFromProbe();
  }
  return _colorCache[cls] || _colorCache['sg-out'] || FALLBACK_COLORS['sg-out'];
}

function readColorsFromProbe() {
  const probe = document.getElementById('sg-color-probe');
  if (!probe) { return FALLBACK_COLORS; }
  const out = {};
  for (const cls of Object.keys(FALLBACK_COLORS)) {
    const el = probe.querySelector('.' + cls);
    if (el) {
      const cs = getComputedStyle(el);
      out[cls] = { bg: cs.backgroundColor || FALLBACK_COLORS[cls].bg,
                   fg: cs.color           || FALLBACK_COLORS[cls].fg };
    } else {
      out[cls] = FALLBACK_COLORS[cls];
    }
  }
  return out;
}

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

// Paint a tile canvas (128×128) with the given background, foreground, and
// centered label. Shared by aperyTile() and the descent animation, which
// repaints the moving tile each time its displayed integer changes.
function paintTileCanvas(canvas, text, cls) {
  const c = COLORS_FOR(cls);
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
}

// Create a flat 1×1 tile with class-colored background and number label,
// lying in the z=0 plane with its lower-left corner at (x, y, 0).
function aperyTile(text, x, y, cls) {
  const canvas = document.createElement('canvas');
  canvas.width = 128;
  canvas.height = 128;
  paintTileCanvas(canvas, text, cls);
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

// Flat grid position for a value: column = residue, row = quotient.
function flatPosOf(val, m) {
  return new THREE.Vector3(val % m + 0.5, Math.floor(val / m) + 0.5, 0);
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

  const W = container.offsetWidth || 600;
  const H = W; // always square

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

  const camera = new THREE.PerspectiveCamera(50, 1, 0.1, extent * 40);
  camera.up.set(0, 0, 1);
  camera.position.set(extent * 1.5, -extent * 1.5, extent * 1.2);

  // ── Lighting ──────────────────────────────────────────────────────────────
  scene.add(new THREE.AmbientLight(0xffffff, 0.55));
  const dirLight = new THREE.DirectionalLight(0xffffff, 0.9);
  dirLight.position.set(extent * 2, -extent * 1.5, extent * 3);
  dirLight.castShadow = true;
  dirLight.shadow.mapSize.width = 1024;
  dirLight.shadow.mapSize.height = 1024;
  dirLight.shadow.camera.near = 0.5;
  dirLight.shadow.camera.far = extent * 20;
  const sc = extent * 2;
  dirLight.shadow.camera.left = -sc;
  dirLight.shadow.camera.right = sc;
  dirLight.shadow.camera.top = sc;
  dirLight.shadow.camera.bottom = -sc;
  scene.add(dirLight);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(W, H);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
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
    const kunzPos = new THREE.Vector3(p.x + 0.5, p.y + 0.5, 0);
    const flatPos = flatPosOf(p.val, m);
    tile.userData = { val: p.val, cls: p.cls, kunzPos, flatPos, isCube: false };
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
      const c = COLORS_FOR(cls);
      matCache[cls] = new THREE.MeshLambertMaterial({
        color: new THREE.Color(c.bg), transparent: true, opacity: 0.85,
      });
    }
    return matCache[cls];
  }

  for (const c of cubes) {
    const mesh = new THREE.Mesh(boxGeo, matFor(c.cls));
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.position.set(c.x + 0.5, c.y + 0.5, c.z);
    const kunzPos = new THREE.Vector3(c.x + 0.5, c.y + 0.5, c.z);
    const flatPos = flatPosOf(c.val, m);
    mesh.userData = { val: c.val, cls: c.cls, kunzPos, flatPos, isCube: true };
    const gap = isGapClass(c.cls);
    mesh.visible = gap ? showGaps : showS;
    scene.add(mesh);
    clickable.push(mesh);
    (gap ? _3dGapMeshes : _3dSMeshes).push(mesh);
  }

  // ── Helper: 3D centre position of any integer value in Kunz coordinates ──
  function posOf(val) {
    const i = ((val % m) + m) % m;
    const ki = (apery[i] - i) / m;
    const z = (val - apery[i]) / m;
    return new THREE.Vector3(i + 0.5, ki + 0.5, z);
  }

  // ── Raycaster: hover highlight + tooltip + click-to-toggle ───────────────
  const raycaster = new THREE.Raycaster();
  const mouse = new THREE.Vector2();
  let hoveredMesh = null;
  let hoveredOriginalMat = null;
  const highlightMat = new THREE.MeshBasicMaterial({ color: 0xff8800, transparent: true, opacity: 0.95 });

  // Thin lines shown while hovering: n→f and f→(f-n).
  let hoverLines = [];

  function clearHoverLines() {
    for (const l of hoverLines) { scene.remove(l); l.geometry.dispose(); }
    hoverLines = [];
  }

  function showHoverLines(n) {
    clearHoverLines();
    const posN = posOf(n);
    const posF = posOf(f);
    if (n !== f) {
      const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints([posN, posF]), _3dHoverLineMat);
      scene.add(line); hoverLines.push(line);
    }
    const complement = f - n;
    if (complement > 0) {
      const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints([posF, posOf(complement)]), _3dHoverLineMat);
      scene.add(line); hoverLines.push(line);
    }
  }

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
    clearHoverLines();
    tooltip.style.display = 'none';
    renderer.domElement.style.cursor = '';
  }

  function setHover(hit, event, { suffix = '', cursor = 'pointer' } = {}) {
    hoveredMesh = hit;
    hoveredOriginalMat = hit.material;
    hit.material = highlightMat;
    const rect = renderer.domElement.getBoundingClientRect();
    tooltip.textContent = `${hit.userData.val} ${CLS_LABEL[hit.userData.cls] || ''}${suffix}`;
    tooltip.style.display = 'block';
    tooltip.style.left = `${event.clientX - rect.left + 12}px`;
    tooltip.style.top = `${event.clientY - rect.top - 20}px`;
    renderer.domElement.style.cursor = cursor;
    showHoverLines(hit.userData.val);
  }

  renderer.domElement.addEventListener('mousemove', event => {
    const hit = hitTest(event);
    if (hit === hoveredMesh) {
      if (hit) {
        const rect = renderer.domElement.getBoundingClientRect();
        tooltip.style.left = `${event.clientX - rect.left + 12}px`;
        tooltip.style.top = `${event.clientY - rect.top - 20}px`;
      }
      return;
    }
    clearHover();
    if (hit) { setHover(hit, event); }
  });

  renderer.domElement.addEventListener('mouseleave', clearHover);

  // Mouse: click toggles. Touch (no hover): first tap inspects, second tap on
  // the same cube within TOUCH_COMMIT_WINDOW commits the toggle.
  const DRAG_THRESHOLD_SQ = 16; // ~4px
  const TOUCH_COMMIT_WINDOW = 1500; // ms
  let pointerDownPos = null;
  let pointerDownType = null;
  let touchSelected = null;
  let touchSelectedAt = 0;

  renderer.domElement.addEventListener('pointerdown', event => {
    pointerDownPos = { x: event.clientX, y: event.clientY };
    pointerDownType = event.pointerType;
  });
  renderer.domElement.addEventListener('pointerup', event => {
    if (!pointerDownPos) { return; }
    const dx = event.clientX - pointerDownPos.x;
    const dy = event.clientY - pointerDownPos.y;
    const type = pointerDownType;
    pointerDownPos = null;
    pointerDownType = null;
    if (dx * dx + dy * dy > DRAG_THRESHOLD_SQ) { return; }
    const hit = hitTest(event);

    if (type === 'touch' && hit && hit.userData.val != null) {
      const now = performance.now();
      const isCommit = touchSelected === hit && (now - touchSelectedAt) < TOUCH_COMMIT_WINDOW;
      if (isCommit) {
        touchSelected = null;
        clearHover();
        if (onToggle) { onToggle(hit.userData.val); }
        return;
      }
      clearHover();
      setHover(hit, event, { suffix: ' — tap again to toggle', cursor: '' });
      touchSelected = hit;
      touchSelectedAt = now;
      return;
    }

    if (hit && hit.userData.val != null && onToggle) {
      onToggle(hit.userData.val);
    } else if (!hit) {
      // Empty-space click: toggle flatten animation
      touchSelected = null;
      clearHover();
      if (_3dFlatState === 'kunz') { _3dFlatState = 'to-flat'; _3dFlatT = 0; }
      else if (_3dFlatState === 'flat') { _3dFlatState = 'to-kunz'; _3dFlatT = 0; }
      else if (_3dFlatState === 'to-flat') { _3dFlatState = 'to-kunz'; _3dFlatT = 1 - _3dFlatT; }
      else if (_3dFlatState === 'to-kunz') { _3dFlatState = 'to-flat'; _3dFlatT = 1 - _3dFlatT; }
    }
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

  // Reset flatten state on each render3d call
  _3dFlatState = 'kunz';
  _3dFlatT = 0;
  // Expose for descent/ascent animations queued from app.js.
  _3dClickable = clickable;
  _3dCurrentM = m;
  // Drop any pending animation tied to the previous scene (its meshes are gone).
  if (_3dPendingAnim?.onDone) { _3dPendingAnim.onDone(); }
  _3dPendingAnim = null;
  let lastTime = performance.now();

  function animate(now) {
    _3dAnimId = requestAnimationFrame(animate);
    const dt = (now - lastTime) / 1000;
    lastTime = now;

    // Flatten/unflatten animation
    if (_3dFlatState === 'to-flat' || _3dFlatState === 'to-kunz') {
      _3dFlatT = Math.min(1, _3dFlatT + dt / FLAT_DURATION);
      // Smooth easing (ease in-out)
      const ease = _3dFlatT < 0.5
        ? 2 * _3dFlatT * _3dFlatT
        : 1 - 2 * (1 - _3dFlatT) * (1 - _3dFlatT);
      const t = _3dFlatState === 'to-flat' ? ease : 1 - ease;
      for (const mesh of clickable) {
        const {kunzPos, flatPos, isCube} = mesh.userData;
        mesh.position.lerpVectors(kunzPos, flatPos, t);
        // Cubes shrink z-height to a thin disc when flat
        if (isCube) { mesh.scale.z = 1 - t * 0.95; }
      }
      if (_3dFlatT >= 1) {
        _3dFlatState = _3dFlatState === 'to-flat' ? 'flat' : 'kunz';
        _3dFlatT = 0;
      }
    }

    // Descent / ascent animation step. Both run the same two-phase loop:
    //   phase 1: tile slides one z toward the new Apéry slot; descent's gap
    //     cube explodes; descent's below-cubes follow the tile down.
    //   phase 2: every survivor lerps to its precomputed `finalPos`, the exact
    //     slot the new render3d will draw — so the swap is a colour flip.
    // Direction-specific bits live in the column entries and labelDir.
    if (_3dPendingAnim) {
      const a = _3dPendingAnim;
      a.t = Math.min(1, a.t + dt / a.duration);
      const PHASE1 = 0.6;
      if (a.t <= PHASE1) {
        const t1 = a.t / PHASE1;
        const e1 = 1 - (1 - t1) * (1 - t1);
        for (const e of a.column) {
          if (e.category === 'gap') {
            e.mesh.scale.setScalar(1 + e1 * 1.8);
            e.mesh.material.opacity = (1 - e1) * 0.85;
          } else if (e.movesPhase1) {
            e.mesh.position.lerpVectors(e.start, e.p1End, e1);
          }
        }
        if (a.tileMesh) {
          // Tick the label toward val ± m in m+1 equal segments.
          const step = Math.min(a.labelStep, Math.floor(e1 * (a.labelStep + 1)));
          const want = a.startVal + a.labelDir * step;
          if (want !== a.shownVal) {
            a.shownVal = want;
            paintTileCanvas(a.tileCanvas, String(want), a.tileMesh.userData.cls);
            a.tileMesh.material.map.needsUpdate = true;
          }
        }
      } else {
        const t2 = (a.t - PHASE1) / (1 - PHASE1);
        const e2 = 1 - (1 - t2) * (1 - t2);
        for (const e of a.column) {
          if (e.category === 'gap') { continue; }
          e.mesh.position.lerpVectors(e.p1End, e.finalPos, e2);
        }
      }
      if (a.t >= 1) {
        // Descent's gap cube has a cloned MeshLambertMaterial — dispose it so
        // we don't leak GPU programs across the loop's ≤500 steps.
        if (a.gapMesh) { a.gapMesh.material.dispose(); }
        const done = a.onDone;
        _3dPendingAnim = null;
        done();
      }
    }

    controls.update();
    renderer.render(scene, camera);
  }
  animate(performance.now());
}

// Look up a mesh in the current scene by its numeric value.
function findMeshByVal(val) {
  return _3dClickable.find(mesh => mesh.userData.val === val) || null;
}

// Resolve immediately when no rAF loop is processing animations (e.g. before
// the first render3d call, or the 3D scene was never built).
function loopIsLive() { return _3dAnimId !== null; }

// Common column-builder for both directions. `dir` is +1 (descent) or -1
// (ascent); positions are computed so every survivor's finalPos is the exact
// slot the new render3d will draw, regardless of the phase-1 motion. The new
// Apéry's y sits at start.y - dir.
//
// Categories:
//   'tile'   — the moving Apéry tile (val=target); relabel ticks toward
//              val±m so it lands as the new Apéry.
//   'gap'    — descent only: the cube at val-m that explodes (no ascent
//              analog, since val+m > f always for ascent targets).
//   'below'  — cubes at val-m (ascent), val-2m, val-3m, … (gaps under tile).
//   'above'  — sporadic S-cubes (val+m, …); for both directions val±m > f,
//              so this is never hit on real inputs — kept defensively.
//
// `start` is cloned because lerpVectors(start, …) is called with this===mesh.position
// in phase 1; the clone keeps the source point fixed across frames.
function buildColumn(val, m, dir) {
  const residue = val % m;
  const column = [];
  for (const mesh of _3dClickable) {
    const v = mesh.userData.val;
    if (v == null) { continue; }
    if (v % m !== residue) { continue; }
    const p = mesh.position;
    const start = p.clone();
    let category, p1End, finalPos, movesPhase1 = false;
    if (v === val) {
      category = 'tile';
      p1End = new THREE.Vector3(p.x, p.y, p.z - dir);
      finalPos = new THREE.Vector3(p.x, p.y - dir, p.z);
      movesPhase1 = true;
    } else if (dir === 1 && v === val - m) {
      category = 'gap';
      p1End = start;
      finalPos = start;
    } else if (v < val && dir === 1) {
      category = 'below';
      p1End = new THREE.Vector3(p.x, p.y, p.z - 1); // descent: follow tile down
      finalPos = new THREE.Vector3(p.x, p.y - 1, p.z + 1);
      movesPhase1 = true;
    } else {
      // Ascent's below cubes (v < val) or descent's defensive above (v > val):
      // no phase-1 motion; phase 2 carries them onto the new grid.
      category = v < val ? 'below' : 'above';
      p1End = start;
      finalPos = new THREE.Vector3(p.x, p.y - dir, p.z + dir);
    }
    column.push({ mesh, category, start, p1End, finalPos, movesPhase1 });
  }
  return column;
}

export function animate3dDescent(val) {
  return new Promise(resolve => {
    if (!loopIsLive()) { resolve(); return; }
    const m = _3dCurrentM;
    const tileMesh = findMeshByVal(val);
    const gapMesh = findMeshByVal(val - m);
    if (!tileMesh && !gapMesh) { resolve(); return; }
    const column = buildColumn(val, m, 1);
    if (gapMesh) {
      // Cube materials are shared via matCache — clone so we don't fade every
      // cube of the same class.
      gapMesh.material = gapMesh.material.clone();
      gapMesh.material.transparent = true;
    }
    const tileCanvas = tileMesh ? tileMesh.material.map.image : null;
    _3dPendingAnim = {
      type: 'descent',
      t: 0,
      duration: STEP_DURATION,
      tileMesh,
      gapMesh,
      tileCanvas,
      startVal: val,
      labelStep: m,
      labelDir: -1,
      shownVal: val,
      column,
      onDone: resolve,
    };
  });
}

// Animate an ascent step on the min-gen `val`: lift its tile by Δz=+1 with
// the label ticking val → val+m so it lands as the new Apéry one Kunz row up.
// Phase 2 shifts every survivor (Δy=+1, Δz=-1) onto the new grid. No cube
// explodes — val+m > f, so the position above the tile is empty in the old view.
export function animate3dAscent(val) {
  return new Promise(resolve => {
    if (!loopIsLive()) { resolve(); return; }
    const m = _3dCurrentM;
    const tileMesh = findMeshByVal(val);
    if (!tileMesh) { resolve(); return; }
    const column = buildColumn(val, m, -1);
    const tileCanvas = tileMesh.material.map.image;
    _3dPendingAnim = {
      type: 'ascent',
      t: 0,
      duration: STEP_DURATION,
      tileMesh,
      tileCanvas,
      startVal: val,
      labelStep: m,
      labelDir: +1,
      shownVal: val,
      column,
      onDone: resolve,
    };
  });
}

// Keep the 3D canvas square and resize the renderer when the wrapper is dragged.
new ResizeObserver(entries => {
  const w = Math.round(entries[0].contentRect.width);
  const container = document.getElementById('sg-3d-container');
  container.style.height = `${w}px`;
  if (_3dRenderer && _3dCamera) {
    _3dRenderer.setSize(w, w);
    _3dCamera.aspect = 1;
    _3dCamera.updateProjectionMatrix();
  }
}).observe(document.getElementById('sg-3d-wrapper'));
