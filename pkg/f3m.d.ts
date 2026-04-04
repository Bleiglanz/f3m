/* tslint:disable */
/* eslint-disable */

export class JsSemigroup {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    add_all_pf(): Uint32Array;
    add_reflected_gaps(): Uint32Array;
    /**
     * Returns `true` if the semigroup has a generator coprime to m (i.e. self-gluing is possible).
     */
    can_self_glue(): boolean;
    is_element(x: number): boolean;
    kunz(i: number, j: number): number;
    s_over_2(): Uint32Array;
    /**
     * Returns the generators of the self-gluing of this semigroup (α = m, β = first
     * generator coprime to m), or an empty vec if no such generator exists.
     */
    self_glue(): Uint32Array;
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
    readonly special_pf_str: string[];
    readonly type_t: number;
    readonly wilf: number;
}

/**
 * Build the full combined table: structure grid + repeated header + Apéry row + Kunz matrix.
 * When `tilt == 0` columns span `[0, m)`; when `tilt != 0` they span `[-2m, 2m)` so
 * the wider neighbourhood is visible for a tilted view.
 */
export function combined_table(s: JsSemigroup, offset: number, tilt: number, show_kunz: boolean): string;

/**
 * Replace a[expr], q[expr] and scalars in `expr` with semigroup values:
 *   a[i] → i-th Apéry number (0 if i≥m),  q[i] → i-th generator (0 if i≥e)
 *   e=embedding dim, g=gaps, f=Frobenius, t=type, m=multiplicity,
 *   Q=largest generator (max gen), A=max Apéry element (= f+m),
 *   s=σ (elements below conductor), b=reflected gap count
 * Index expressions are evaluated recursively. Returns None on any error.
 */
export function eval_expr(expr: string, s: JsSemigroup): number | undefined;

export function gap_footer(): string;

export function gap_header(): string;

/**
 * Returns an HTML table mapping each integer 0..=f+m to its classification,
 * with a third "Diff" column showing SPF generator-difference expressions.
 */
export function js_classify_table(s: JsSemigroup): string;

/**
 * Returns the set-containment relationship between two semigroups as a symbol:
 * "⊂" (s1 ⊊ s2), "=" (equal), "⊃" (s1 ⊋ s2), or "?" (incomparable).
 */
export function js_cmp_semigroups(s1: JsSemigroup, s2: JsSemigroup): string;

export function js_compute(input: string): JsSemigroup;

/**
 * Return the GAP assertion block for a single semigroup, numbered `idx`.
 */
export function js_gap_block(s: JsSemigroup, idx: number): string;

/**
 * Edges as a flat [from, to, from, to, ...] u32 array for 0..=upto.
 */
export function js_graph_edge_pairs(s: JsSemigroup, upto: number): Uint32Array;

/**
 * Graph edges up to `upto` as plain text pairs, one per line.
 */
export function js_graph_edges_text(s: JsSemigroup, upto: number): string;

/**
 * Node IDs (as u32) that appear in the graph for 0..=upto.
 */
export function js_graph_node_ids(s: JsSemigroup, upto: number): Uint32Array;

/**
 * CSS class name for node `n` using the same classification as the combined table.
 */
export function js_node_class(s: JsSemigroup, n: number): string;

/**
 * Compact summary row for the properties table: nested table with header + one data row.
 */
export function shortprop(s: JsSemigroup): string;

/**
 * Flat `<td>` cells for use in the history table row (no nested table, no header).
 */
export function shortprop_tds(s: JsSemigroup): string;

/**
 * Containment-comparison HTML symbol between `history[a]` and `history[b]`.
 */
export function state_cmp(a: number, b: number): string;

/**
 * Current history index, or -1 if history is empty.
 */
export function state_current_idx(): number;

/**
 * Full GAP script: header + all accumulated blocks + footer.
 */
export function state_gap_output(): string;

/**
 * Return the semigroup at history index `idx`.
 */
export function state_get(idx: number): JsSemigroup;

/**
 * Get the evaluator expression string.
 */
export function state_get_eva_expr(): string;

/**
 * Get/set `show_classification` display toggle.
 */
export function state_get_show_classification(): boolean;

/**
 * Get/set `show_gaps` display toggle.
 */
export function state_get_show_gaps(): boolean;

/**
 * Get/set `show_kunz` display toggle.
 */
export function state_get_show_kunz(): boolean;

/**
 * Get/set `show_s` display toggle.
 */
export function state_get_show_s(): boolean;

/**
 * Number of semigroups in history.
 */
export function state_len(): number;

/**
 * Compute a semigroup from comma-separated input, push it to history,
 * update `current_idx`, and return the new index.
 */
export function state_push(input: string): number;

/**
 * Set the current history index (call when the user re-focuses a history entry).
 */
export function state_set_current_idx(idx: number): void;

/**
 * Set the evaluator expression string.
 */
export function state_set_eva_expr(expr: string): void;

export function state_set_show_classification(v: boolean): void;

export function state_set_show_gaps(v: boolean): void;

export function state_set_show_kunz(v: boolean): void;

export function state_set_show_s(v: boolean): void;

/**
 * Pure x-y grid for the Tilt tab: no Apéry row, no Kunz matrix.
 * x (columns) and y (rows) both run from -3m to 3m.
 * y increases upward (highest y at top). x increases left to right.
 * Element at (x, y) = y*m + x - tilt*y.
 */
export function tilt_table(s: JsSemigroup, tilt: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_jssemigroup_free: (a: number, b: number) => void;
    readonly gap_footer: () => [number, number];
    readonly gap_header: () => [number, number];
    readonly js_classify_table: (a: number) => [number, number];
    readonly js_cmp_semigroups: (a: number, b: number) => [number, number];
    readonly js_compute: (a: number, b: number) => number;
    readonly js_gap_block: (a: number, b: number) => [number, number];
    readonly jssemigroup_add_all_pf: (a: number) => [number, number];
    readonly jssemigroup_add_reflected_gaps: (a: number) => [number, number];
    readonly jssemigroup_apery_set: (a: number) => [number, number];
    readonly jssemigroup_blob: (a: number) => [number, number];
    readonly jssemigroup_can_self_glue: (a: number) => number;
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
    readonly jssemigroup_s_over_2: (a: number) => [number, number];
    readonly jssemigroup_self_glue: (a: number) => [number, number];
    readonly jssemigroup_special_pf: (a: number) => [number, number];
    readonly jssemigroup_special_pf_str: (a: number) => [number, number];
    readonly jssemigroup_toggle: (a: number, b: number) => number;
    readonly jssemigroup_type_t: (a: number) => number;
    readonly jssemigroup_wilf: (a: number) => number;
    readonly eval_expr: (a: number, b: number, c: number) => number;
    readonly tilt_table: (a: number, b: number) => [number, number];
    readonly js_graph_edge_pairs: (a: number, b: number) => [number, number];
    readonly js_graph_edges_text: (a: number, b: number) => [number, number];
    readonly js_graph_node_ids: (a: number, b: number) => [number, number];
    readonly js_node_class: (a: number, b: number) => [number, number];
    readonly state_cmp: (a: number, b: number) => [number, number];
    readonly state_gap_output: () => [number, number];
    readonly state_get: (a: number) => number;
    readonly state_get_eva_expr: () => [number, number];
    readonly state_get_show_classification: () => number;
    readonly state_get_show_gaps: () => number;
    readonly state_get_show_kunz: () => number;
    readonly state_get_show_s: () => number;
    readonly state_push: (a: number, b: number) => number;
    readonly state_set_eva_expr: (a: number, b: number) => void;
    readonly state_set_show_classification: (a: number) => void;
    readonly state_set_show_gaps: (a: number) => void;
    readonly state_set_show_kunz: (a: number) => void;
    readonly state_set_show_s: (a: number) => void;
    readonly state_current_idx: () => number;
    readonly state_set_current_idx: (a: number) => void;
    readonly state_len: () => number;
    readonly combined_table: (a: number, b: number, c: number, d: number) => [number, number];
    readonly shortprop: (a: number) => [number, number];
    readonly shortprop_tds: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_drop_slice: (a: number, b: number) => void;
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
