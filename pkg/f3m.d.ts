/* tslint:disable */
/* eslint-disable */

export class JsSemigroup {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    is_element(x: number): boolean;
    kunz(i: number, j: number): number;
    toggle(n: number): JsSemigroup;
    readonly apery_set: Uint32Array;
    readonly blob: Uint32Array;
    readonly count_gap: number;
    readonly count_set: number;
    readonly e: number;
    readonly f: number;
    readonly gen_set: Uint32Array;
    readonly is_symmetric: boolean;
    readonly m: number;
    readonly max_gen: number;
    readonly pf: Uint32Array;
    readonly special_pf: Uint32Array;
    readonly type_t: number;
    readonly wilf: number;
}

/**
 * Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
 * All sections share `m` columns, permuted by `offset` so column `col` shows residue
 * `(offset + col) % m`.
 */
export function combined_table(s: JsSemigroup, offset: number): string;

export function js_compute(input: string): JsSemigroup;

/**
 * Compact summary row for the properties table: nested table with header + one data row.
 */
export function shortprop(s: JsSemigroup): string;

/**
 * Flat `<td>` cells for use in the history table row (no nested table, no header).
 */
export function shortprop_tds(s: JsSemigroup): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_jssemigroup_free: (a: number, b: number) => void;
    readonly js_compute: (a: number, b: number) => number;
    readonly jssemigroup_apery_set: (a: number) => [number, number];
    readonly jssemigroup_blob: (a: number) => [number, number];
    readonly jssemigroup_count_gap: (a: number) => number;
    readonly jssemigroup_count_set: (a: number) => number;
    readonly jssemigroup_e: (a: number) => number;
    readonly jssemigroup_f: (a: number) => number;
    readonly jssemigroup_gen_set: (a: number) => [number, number];
    readonly jssemigroup_is_element: (a: number, b: number) => number;
    readonly jssemigroup_is_symmetric: (a: number) => number;
    readonly jssemigroup_kunz: (a: number, b: number, c: number) => number;
    readonly jssemigroup_m: (a: number) => number;
    readonly jssemigroup_max_gen: (a: number) => number;
    readonly jssemigroup_pf: (a: number) => [number, number];
    readonly jssemigroup_special_pf: (a: number) => [number, number];
    readonly jssemigroup_toggle: (a: number, b: number) => number;
    readonly jssemigroup_type_t: (a: number) => number;
    readonly jssemigroup_wilf: (a: number) => number;
    readonly combined_table: (a: number, b: number) => [number, number];
    readonly shortprop: (a: number) => [number, number];
    readonly shortprop_tds: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
