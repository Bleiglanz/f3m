/* @ts-self-types="./f3m.d.ts" */

export class JsSemigroup {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JsSemigroup.prototype);
        obj.__wbg_ptr = ptr;
        JsSemigroupFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsSemigroupFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jssemigroup_free(ptr, 0);
    }
    /**
     * @returns {Uint32Array}
     */
    get apery_set() {
        const ret = wasm.jssemigroup_apery_set(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    get blob() {
        const ret = wasm.jssemigroup_blob(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    get count_gap() {
        const ret = wasm.jssemigroup_count_gap(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get count_set() {
        const ret = wasm.jssemigroup_count_set(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get e() {
        const ret = wasm.jssemigroup_e(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get f() {
        const ret = wasm.jssemigroup_f(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint32Array}
     */
    get gen_set() {
        const ret = wasm.jssemigroup_gen_set(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} x
     * @returns {boolean}
     */
    is_element(x) {
        const ret = wasm.jssemigroup_is_element(this.__wbg_ptr, x);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get is_symmetric() {
        const ret = wasm.jssemigroup_is_symmetric(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} i
     * @param {number} j
     * @returns {number}
     */
    kunz(i, j) {
        const ret = wasm.jssemigroup_kunz(this.__wbg_ptr, i, j);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get m() {
        const ret = wasm.jssemigroup_m(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get max_gen() {
        const ret = wasm.jssemigroup_max_gen(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint32Array}
     */
    get pf() {
        const ret = wasm.jssemigroup_pf(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    get special_pf() {
        const ret = wasm.jssemigroup_special_pf(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string[]}
     */
    get special_pf_str() {
        const ret = wasm.jssemigroup_special_pf_str(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} n
     * @returns {JsSemigroup}
     */
    toggle(n) {
        const ret = wasm.jssemigroup_toggle(this.__wbg_ptr, n);
        return JsSemigroup.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    get type_t() {
        const ret = wasm.jssemigroup_type_t(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get wilf() {
        const ret = wasm.jssemigroup_wilf(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) JsSemigroup.prototype[Symbol.dispose] = JsSemigroup.prototype.free;

/**
 * Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
 * When `tilt == 0` columns span `[0, m)`; when `tilt != 0` they span `[-2m, 2m)` so
 * the wider neighbourhood is visible for a tilted view.
 * @param {JsSemigroup} s
 * @param {number} offset
 * @param {number} tilt
 * @returns {string}
 */
export function combined_table(s, offset, tilt) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.combined_table(s.__wbg_ptr, offset, tilt);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Replace a[expr], q[expr] and scalars in `expr` with semigroup values:
 *   a[i] → i-th Apéry number (0 if i≥m),  q[i] → i-th generator (0 if i≥e)
 *   e=embedding dim, g=gaps, f=Frobenius, t=type, m=multiplicity,
 *   Q=largest generator (max gen), A=max Apéry element (= f+m)
 * Index expressions are evaluated recursively. Returns None on any error.
 * @param {string} expr
 * @param {JsSemigroup} s
 * @returns {number | undefined}
 */
export function eval_expr(expr, s) {
    const ptr0 = passStringToWasm0(expr, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    _assertClass(s, JsSemigroup);
    const ret = wasm.eval_expr(ptr0, len0, s.__wbg_ptr);
    return ret === 0x100000001 ? undefined : ret;
}

/**
 * @returns {string}
 */
export function gap_footer() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.gap_footer();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {string}
 */
export function gap_header() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.gap_header();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Returns an HTML table mapping each integer 0..=f+m to its classification.
 * The first column uses the same colour+toggle span as the structure grid.
 * @param {JsSemigroup} s
 * @returns {string}
 */
export function js_classify_table(s) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.js_classify_table(s.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Returns the set-containment relationship between two semigroups as a symbol:
 * "⊂" (s1 ⊊ s2), "=" (equal), "⊃" (s1 ⊋ s2), or "?" (incomparable).
 * @param {JsSemigroup} s1
 * @param {JsSemigroup} s2
 * @returns {string}
 */
export function js_cmp_semigroups(s1, s2) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s1, JsSemigroup);
        _assertClass(s2, JsSemigroup);
        const ret = wasm.js_cmp_semigroups(s1.__wbg_ptr, s2.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} input
 * @returns {JsSemigroup}
 */
export function js_compute(input) {
    const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.js_compute(ptr0, len0);
    return JsSemigroup.__wrap(ret);
}

/**
 * Return the GAP assertion block for a single semigroup, numbered `idx`.
 * @param {JsSemigroup} s
 * @param {number} idx
 * @returns {string}
 */
export function js_gap_block(s, idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.js_gap_block(s.__wbg_ptr, idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Edges as a flat [from, to, from, to, ...] u32 array for 0..=upto.
 * @param {JsSemigroup} s
 * @param {number} upto
 * @returns {Uint32Array}
 */
export function js_graph_edge_pairs(s, upto) {
    _assertClass(s, JsSemigroup);
    const ret = wasm.js_graph_edge_pairs(s.__wbg_ptr, upto);
    var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

/**
 * Graph edges up to `upto` as plain text pairs, one per line.
 * @param {JsSemigroup} s
 * @param {number} upto
 * @returns {string}
 */
export function js_graph_edges_text(s, upto) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.js_graph_edges_text(s.__wbg_ptr, upto);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Node IDs (as u32) that appear in the graph for 0..=upto.
 * @param {JsSemigroup} s
 * @param {number} upto
 * @returns {Uint32Array}
 */
export function js_graph_node_ids(s, upto) {
    _assertClass(s, JsSemigroup);
    const ret = wasm.js_graph_node_ids(s.__wbg_ptr, upto);
    var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

/**
 * CSS class name for node `n` using the same classification as the combined table.
 * @param {JsSemigroup} s
 * @param {number} n
 * @returns {string}
 */
export function js_node_class(s, n) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.js_node_class(s.__wbg_ptr, n);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Compact summary row for the properties table: nested table with header + one data row.
 * @param {JsSemigroup} s
 * @returns {string}
 */
export function shortprop(s) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.shortprop(s.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Flat `<td>` cells for use in the history table row (no nested table, no header).
 * @param {JsSemigroup} s
 * @returns {string}
 */
export function shortprop_tds(s) {
    let deferred1_0;
    let deferred1_1;
    try {
        _assertClass(s, JsSemigroup);
        const ret = wasm.shortprop_tds(s.__wbg_ptr);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./f3m_bg.js": import0,
    };
}

const JsSemigroupFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jssemigroup_free(ptr >>> 0, 1));

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('f3m_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
