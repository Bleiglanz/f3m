// Three.js renderer state — kept module-level so render3d() can clean up on re-render.
let _3dAnimId = null; // requestAnimationFrame handle
let _3dRenderer = null; // WebGLRenderer instance

// Find (x, y) with x >= 0, y >= 0 such that x*g1 + y*g2 = val, or null.
function factorize(val, g1, g2) {
  for (let y = 0; y * g2 <= val; y++) {
    const rem = val - y * g2;
    if (rem % g1 === 0) { return { x: rem / g1, y }; }
  }
  return null;
}

// Create a flat 1x1 plane mesh with a blue background and white number label,
// lying in the z=0 plane with its lower-left corner at (x, y, 0).
function aperyTile(text, x, y) {
  const canvas = document.createElement('canvas');
  canvas.width = 128;
  canvas.height = 128;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#2266cc';
  ctx.fillRect(0, 0, 128, 128);
  ctx.strokeStyle = '#fff';
  ctx.lineWidth = 3;
  ctx.strokeRect(1, 1, 126, 126);
  ctx.font = 'bold 56px sans-serif';
  ctx.fillStyle = '#fff';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 64, 64);
  const texture = new THREE.CanvasTexture(canvas);
  const geo = new THREE.PlaneGeometry(1, 1);
  const mat = new THREE.MeshBasicMaterial({ map: texture, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(geo, mat);
  // PlaneGeometry faces +z by default; position centre at (x+0.5, y+0.5, 0)
  mesh.position.set(x + 0.5, y + 0.5, 0);
  return mesh;
}

// Create a text-only sprite for axis labels.
function textSprite(text, color, size) {
  const canvas = document.createElement('canvas');
  canvas.width = 128;
  canvas.height = 64;
  const ctx = canvas.getContext('2d');
  ctx.font = 'bold 40px sans-serif';
  ctx.fillStyle = color;
  ctx.textAlign = 'center';
  ctx.fillText(text, 64, 46);
  const texture = new THREE.CanvasTexture(canvas);
  const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture }));
  sprite.scale.set(size, size * 0.5, 1);
  return sprite;
}

// (Re-)render the 3D view for a semigroup with embedding dimension 3.
//
// Axes: x >= 0 (gen[1]), y >= 0 (gen[2]), z (gen[0]=m, both directions).
// z = 0 plane: blue tiles at Apery-element (x,y) positions, labelled with the value.
// z < 0: gaps (green cubes); Frobenius number is red.
// z > 0: sporadic elements of S below the conductor (dark cubes).
export function render3d(s) {
  const container = document.getElementById('sg-3d-container');
  container.innerHTML = '';
  if (_3dAnimId) { cancelAnimationFrame(_3dAnimId); _3dAnimId = null; }
  if (_3dRenderer) { _3dRenderer.dispose(); _3dRenderer = null; }

  if (s.e !== 3) {
    container.textContent = '3D view is only available for embedding dimension e = 3.';
    return;
  }

  const gens = Array.from(s.gen_set); // [m, g1, g2]
  const m = gens[0];
  const g1 = gens[1];
  const g2 = gens[2];
  const {f} = s;
  const apery = Array.from(s.apery_set); // apery[i] = smallest element ≡ i (mod m)

  // For each Apery element, find its (x,y) factorization and collect cubes along the z-column.
  // z < 0: gaps (green, Frobenius red)   z = 0: Apery tile   z > 0: sporadic elements of S (dark)
  // By minimality of Apery elements: a - k·m is a gap for all k >= 1 with a - k·m > 0,
  // and a + k·m is in S for all k >= 1. We cap at f+1 (conductor).
  const aperyPoints = []; // { x, y, val }
  const cubes = [];       // { x, y, z, color }

  for (let i = 0; i < m; i++) {
    const a = apery[i];
    const pt = factorize(a, g1, g2);
    if (!pt) { continue; }
    aperyPoints.push({ x: pt.x, y: pt.y, val: a });

    // z < 0: gaps (a - k·m > 0 are all gaps by Apery minimality)
    for (let z = -1; ; z--) {
      const val = a + z * m;
      if (val <= 0) { break; }
      cubes.push({ x: pt.x, y: pt.y, z, color: val === f ? 0xdd0000 : 0x44aa44 });
    }

    // z > 0: sporadic elements of S (below conductor f+1)
    for (let z = 1; ; z++) {
      const val = a + z * m;
      if (val > f) { break; }
      cubes.push({ x: pt.x, y: pt.y, z, color: 0x222222 });
    }
  }

  const W = container.offsetWidth || 800;
  const H = 500;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xfafafa);

  // Determine scene extent from data
  let xMax = 1, yMax = 1, zMin = 0, zMax = 0;
  for (const p of aperyPoints) {
    if (p.x > xMax) { xMax = p.x; }
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

  // ── Axes ─────────────────────────────────────────────────────────────────
  const axisLen = extent + 2;
  const axisMat = new THREE.LineMaterial({ color: 0x111111, linewidth: 2 });
  axisMat.resolution.set(W, H);

  const xGeo = new THREE.LineGeometry();
  xGeo.setPositions([0, 0, 0, axisLen, 0, 0]);
  scene.add(new THREE.Line2(xGeo, axisMat));

  const yGeo = new THREE.LineGeometry();
  yGeo.setPositions([0, 0, 0, 0, axisLen, 0]);
  scene.add(new THREE.Line2(yGeo, axisMat));

  const zAxisMat = new THREE.LineMaterial({ color: 0x0000ff, linewidth: 2 });
  zAxisMat.resolution.set(W, H);
  const zGeo = new THREE.LineGeometry();
  zGeo.setPositions([0, 0, zMin, 0, 0, zMax]);
  scene.add(new THREE.Line2(zGeo, zAxisMat));

  // Arrowheads
  const arrowLen = extent * 0.06;
  const arrowRad = arrowLen * 0.4;
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(1, 0, 0), new THREE.Vector3(axisLen, 0, 0), arrowLen, 0x111111, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, axisLen, 0), arrowLen, 0x111111, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, zMax), arrowLen, 0x0000ff, arrowLen, arrowRad));
  scene.add(new THREE.ArrowHelper(new THREE.Vector3(0, 0, -1), new THREE.Vector3(0, 0, zMin), arrowLen, 0x0000ff, arrowLen, arrowRad));

  // Axis labels
  const labelSize = extent * 0.15;
  const xLabel = textSprite(`g\u2081=${g1}`, '#111', labelSize);
  xLabel.position.set(axisLen + extent * 0.1, 0, 0);
  scene.add(xLabel);
  const yLabel = textSprite(`g\u2082=${g2}`, '#111', labelSize);
  yLabel.position.set(0, axisLen + extent * 0.1, 0);
  scene.add(yLabel);
  const zLabel = textSprite(`m=${m}`, '#00c', labelSize);
  zLabel.position.set(0, 0, zMax + extent * 0.12);
  scene.add(zLabel);

  // ── Apery tiles (flat blue squares with number) in the z=0 plane ────────
  for (const p of aperyPoints) {
    scene.add(aperyTile(String(p.val), p.x, p.y));
  }

  // ── Hyperplanes x+y+z = 0 and x+y+z = f+1 (transparent) ─────────────────
  // Each plane is clipped to the visible bounding box [0,xMax+1] × [0,yMax+1] × [zMin,zMax].
  // For a plane x+y+z=k, at each (x,y) corner of the bbox, z = k - x - y.
  // We collect the polygon of intersection with the bbox and triangulate it.
  const bx = xMax + 1, by = yMax + 1;
  function planeMesh(k, color) {
    // Vertices of the bbox-plane intersection: walk bbox edges, collect points where k-x-y is in [zMin,zMax].
    const pts = [];
    // For each corner (cx,cy) of the xy-rect, compute z = k - cx - cy
    const corners = [[0, 0], [bx, 0], [bx, by], [0, by]];
    const edges = [[0, 1], [1, 2], [2, 3], [3, 0]];
    // Collect points on the xy-boundary where z is in range, plus z-clipped points.
    for (let ei = 0; ei < 4; ei++) {
      const [i0, i1] = edges[ei];
      const [x0, y0] = corners[i0];
      const [x1, y1] = corners[i1];
      const z0 = k - x0 - y0, z1 = k - x1 - y1;
      // Add start corner if z in range
      if (z0 >= zMin && z0 <= zMax) { pts.push(new THREE.Vector3(x0, y0, z0)); }
      // Clip against zMin
      for (const zClip of [zMin, zMax]) {
        // t where k - lerp(x,y) = zClip → t = (z0 - zClip) / (z0 - z1)
        if ((z0 - zClip) * (z1 - zClip) < 0) {
          const t = (z0 - zClip) / (z0 - z1);
          pts.push(new THREE.Vector3(x0 + t * (x1 - x0), y0 + t * (y1 - y0), zClip));
        }
      }
    }
    // Also add points on z=zMin and z=zMax bbox edges (horizontal rectangle edges at z extremes)
    // For z = zMin: x + y = k - zMin, line in the xy plane; clip to [0,bx]×[0,by]
    for (const zClip of [zMin, zMax]) {
      const s = k - zClip; // x + y = s
      // Intersect x+y=s with the xy-rect boundary
      const seg = [];
      if (s >= 0 && s <= by) { seg.push(new THREE.Vector3(0, s, zClip)); }        // x=0
      if (s >= 0 && s <= bx) { seg.push(new THREE.Vector3(s, 0, zClip)); }        // y=0
      if (s - bx >= 0 && s - bx <= by) { seg.push(new THREE.Vector3(bx, s - bx, zClip)); } // x=bx
      if (s - by >= 0 && s - by <= bx) { seg.push(new THREE.Vector3(s - by, by, zClip)); } // y=by
      for (const p of seg) { pts.push(p); }
    }
    if (pts.length < 3) { return null; }
    // Sort points by angle around centroid, projected onto the plane x+y+z=k.
    // Plane normal is (1,1,1)/√3. Use two orthogonal in-plane axes to get 2D angles.
    const cx = pts.reduce((s, p) => s + p.x, 0) / pts.length;
    const cy = pts.reduce((s, p) => s + p.y, 0) / pts.length;
    const cz = pts.reduce((s, p) => s + p.z, 0) / pts.length;
    // u = normalise(1,-1,0), v = normalise(1,1,-2)
    const sq2 = Math.SQRT2, sq6 = Math.sqrt(6);
    pts.sort((a, b) => {
      const adx = a.x - cx, ady = a.y - cy, adz = a.z - cz;
      const bdx = b.x - cx, bdy = b.y - cy, bdz = b.z - cz;
      const au = (adx - ady) / sq2, av = (adx + ady - 2 * adz) / sq6;
      const bu = (bdx - bdy) / sq2, bv = (bdx + bdy - 2 * bdz) / sq6;
      return Math.atan2(av, au) - Math.atan2(bv, bu);
    });
    // Remove near-duplicates
    const eps = 1e-6;
    const unique = [pts[0]];
    for (let i = 1; i < pts.length; i++) {
      if (pts[i].distanceTo(unique[unique.length - 1]) > eps) { unique.push(pts[i]); }
    }
    if (unique.length < 3) { return null; }
    // Fan triangulation from centroid
    const geo = new THREE.BufferGeometry();
    const verts = [];
    const center = new THREE.Vector3(cx, cy, cz);
    for (let i = 0; i < unique.length; i++) {
      const j = (i + 1) % unique.length;
      verts.push(center.x, center.y, center.z);
      verts.push(unique[i].x, unique[i].y, unique[i].z);
      verts.push(unique[j].x, unique[j].y, unique[j].z);
    }
    geo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
    geo.computeVertexNormals();
    const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.15, side: THREE.DoubleSide, depthWrite: false });
    return new THREE.Mesh(geo, mat);
  }

  const plane0 = planeMesh(0, 0x888888);
  if (plane0) { scene.add(plane0); }
  const planeC = planeMesh(f + 1, 0xff8800);
  if (planeC) { scene.add(planeC); }

  // ── Cubes: green = gap (z<0), red = Frobenius (z<0), dark = sporadic (z>0) ──
  const cubeSize = 0.45;
  const boxGeo = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
  const greenMat = new THREE.MeshBasicMaterial({ color: 0x44aa44, transparent: true, opacity: 0.85 });
  const redMat = new THREE.MeshBasicMaterial({ color: 0xdd0000, transparent: true, opacity: 0.9 });
  const darkMat = new THREE.MeshBasicMaterial({ color: 0x222222, transparent: true, opacity: 0.8 });
  const matMap = { 0x44aa44: greenMat, 0xdd0000: redMat, 0x222222: darkMat };

  for (const c of cubes) {
    const mesh = new THREE.Mesh(boxGeo, matMap[c.color]);
    mesh.position.set(c.x + 0.5, c.y + 0.5, c.z);
    scene.add(mesh);
  }

  // ── Controls & animation loop ────────────────────────────────────────────
  const center = new THREE.Vector3(xMax / 2, yMax / 2, 0);
  const controls = new THREE.OrbitControls(camera, renderer.domElement);
  controls.target.copy(center);
  camera.lookAt(center);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.update();

  function animate() {
    _3dAnimId = requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();
}
