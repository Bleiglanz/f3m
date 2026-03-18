#![warn(clippy::pedantic)]


// compute the greatest common divisor of two numbers
#[must_use] 
pub fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

// compute the greatest common divisor of a vector of numbers
#[must_use] 
pub fn gcd_vec(numbers: &[usize]) -> usize {
    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }
    d
}

//
// simple struct to hold the results
//
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Semigroup {
    // e is the embedding dimension, f is the Frobenius number, m is the multiplicity
    // gen_set is the set of minimal generators, apery_set is the Apery set with respect to m
    // count_set is the number of elements in the semigroup <f - the sporadic elements
    // count_gap is the number of gaps
    // b is the number of reflected gaps (i.e. gaps g such that f-g is also a gap)
    // blob_set is the set of reflected gaps
    pub e: usize,
    pub f: usize,
    pub m: usize,
    pub count_set: usize,
    pub count_gap: usize,
    pub max_gen: usize,
    pub gen_set: Vec<usize>,
    pub apery_set: Vec<usize>,
}

impl Semigroup {
    // check if a number is an element of the semigroup
    #[must_use] 
    pub fn element(&self, x: usize) -> bool {
        let modulus = x % self.m;
        let ap = self.apery_set[modulus];
        x >= ap
    }
    // check if a number is a gap of the semigroup
    #[must_use] 
    pub fn is_gap(&self, x: usize) -> bool {
        !self.element(x)
    }
    // check if symmetric
    /// # Panics
    /// Panics if the symmetric invariant `f + 1 == 2 * g` is violated (internal consistency check).
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        let sym = self.count_gap == self.count_set;
        assert!(!sym || self.f + 1 == 2 * self.count_gap);
        sym
    }
    // compute the wilf quotient
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn wilf(&self) -> f64 {
        let c = self.f as f64 + 1.0f64;
        let spor = self.count_set as f64;
        spor / c
    }
    // check if a number is a reflected gap
    #[must_use] 
    pub fn is_reflected_gap(&self, x: usize) -> bool {
        self.is_gap(x) && self.is_gap(self.f - x)
    }
    // get the blob, the number of reflected gaps
    #[must_use] 
    pub fn blob(&self) -> Vec<usize> {
        (0..self.f).filter(|&x| self.is_reflected_gap(x)).collect()
    }
    // compute the kunz overshoot apery[i]+apery[j]-apery[i+j%m] / m
    // when displayed, it should be a symmetric matix
    // the i-th row sums should be the i-th aperyelement
    /// # Panics
    /// Panics if the Kunz divisibility invariant is violated (internal consistency check).
    #[must_use]
    pub fn kunz(&self, i: usize, j: usize) -> usize {
        let first = i % self.m;
        let second = j % self.m;
        let idx = (i + j) % self.m;
        let sum = self.apery_set[first] + self.apery_set[second];
        assert!(sum >= self.apery_set[idx]);
        let res = sum - self.apery_set[idx];
        assert_eq!(0, res % self.m, "ai+aj-a(i+j) immer duch m teilbar!");
        res / self.m
    }
    //
    // computes PF(S), the set of pseudo-frobenius numers
    // the reflected gaps x such that x+(element of S > 0) is always in S
    // and the type t, the number of this
    //
    // special pseudo frobenius are the ones that are differences
    // of numbers in the genset, i.e. they are in the pft set
    // and are of the form gen[i]-gen[j] where i>j if the gen is sorted
    // and they don't divide f
    //
    #[must_use] 
    pub fn pft(&self) -> ((Vec<usize>, usize), (Vec<usize>, usize)) {
        let mut pf: Vec<usize> = self
            .blob()
            .into_iter()
            .filter(|&g| self.gen_set.iter().all(|&a| self.element(a + g)))
            .collect();
        pf.push(self.f);
        let t = pf.len();
        let normal_pseudofrobenius = (pf, t);
        // Special PF: elements of PF(S) that equal gen[i]-gen[j] (i>j) and don't divide f
        let pf_set: std::collections::HashSet<usize> =
            normal_pseudofrobenius.0.iter().copied().collect();
        let mut special_set = std::collections::HashSet::new();
        for i in 1..self.gen_set.len() {
            for j in 0..i {
                let diff = self.gen_set[i] - self.gen_set[j];
                if pf_set.contains(&diff) && !self.f.is_multiple_of(diff) {
                    special_set.insert(diff);
                }
            }
        }
        let mut special: Vec<usize> = special_set.into_iter().collect();
        special.sort_unstable();
        let st = special.len();
        (normal_pseudofrobenius, (special, st))
    }

    // toggle(self,n): if n is a gap, add it as a generator;
    // if n is a minimal generator, remove it - by removing
    // all n-s for elements of the semigroup from 1..n
    #[must_use] 
    pub fn toggle(&self, n: usize) -> Semigroup {
        if self.is_gap(n) {
            let mut newgen = self.gen_set.clone();
            newgen.push(n);
            compute(&newgen)
        } else {
            let is_newgen = |x: usize| {
                (x > n && self.element(x))
                    || (x < n && self.element(x) && !self.element(n - x))
            };
            let newgen: Vec<usize> = (1..=(self.f + self.m))
                .filter(|&x| is_newgen(x))
                .collect();
            if newgen.is_empty() { return self.clone(); }
            compute(&newgen)
        }
    }
}

//
// compute the numerical semigroup generated by the numbers in the slice "input"
//
// the algorithm is very simple, inspired by wilf ("the circle of lights")
// we just move a window of width 2*(maximal input) along the natural numbers
// Claude: don't touch this function
//
//
/// # Panics
/// Panics if `input` is empty or all-zero (no valid multiplicity).
#[must_use]
#[allow(clippy::cast_possible_wrap)] // window stores usize values as isize sentinels
pub fn compute(input: &[usize]) -> Semigroup {
    let d = gcd_vec(input);
    let mut inputnumbers: Vec<usize> = input.iter().map(|x| x / d).collect();
    inputnumbers.sort_unstable();

    let maximal_input: usize = *inputnumbers.last().unwrap();
    let width = 2 * maximal_input;
    let m: usize = *inputnumbers.first().unwrap();

    let mut aperyset: Vec<usize> = vec![0; m];
    let mut count_set = 1usize; // 0 is already an element in this set
    let mut window = vec![-1isize; width]; // here we store the results
    let mut windowindex = m; // this is the running index
    let mut runlength = 0usize; // number of consecutive hits
    let mut hit: bool = false; // true if the number windowindex is in S
    let mut max_apery: usize = m;
    let mut sum_apery: usize = 0;
    let mut minimal_generators: usize = 1;
    let mut max_atom = m;
    let mut genset: Vec<usize> = Vec::new();
    window[0] = 0;
    let mut i: usize = m; // startindex
    while runlength < m {
        let residue = i % m;
        if 0 == residue {
            // case: a multiple of m
            count_set += 1;
            runlength += 1;
            hit = true;
            window[windowindex] = i as isize;
        } else if aperyset[residue] > 0 && i > aperyset[residue] {
            // case: we already have found an element in this residue class
            count_set += 1;
            runlength += 1;
            hit = true;
            window[windowindex] = i as isize;
        } else {
            // ok, we must ckeck this number by going back to windowindex-generator for all generators
            for k in &inputnumbers[1..] {
                if windowindex >= *k && window[windowindex - k] >= 0 {
                    // case window[windowindex - k] is already an element of S
                    count_set += 1;
                    runlength += 1;
                    hit = true;
                    window[windowindex] = i as isize;
                    aperyset[residue] = i;
                    sum_apery += i;
                    if i > max_apery {
                        max_apery = i;
                    }
                    if 0 == window[windowindex - *k] {
                        minimal_generators += 1;
                        genset.push(i);
                        if max_atom < i {
                            max_atom = i;
                        }
                    }
                    break;
                }
            }
        }
        if !hit {
            runlength = 0;
        }
        hit = false;
        i += 1;
        //
        // copy the right half of the window to the left and continue
        //
        if windowindex == width - 1 {
            let (dst, src) = window.split_at_mut(maximal_input);
            dst[0..maximal_input].clone_from_slice(&src[..maximal_input]);
            windowindex = maximal_input;
        } else {
            windowindex += 1;
        }
    }
    genset.push(m);
    assert_eq!(genset.len(), minimal_generators);
    genset.sort_unstable();
    assert_eq!(aperyset.len(), m);

    Semigroup {
        e: minimal_generators,
        f: max_apery - m,
        m,
        count_set: count_set - m,
        count_gap: (sum_apery - ((m - 1) * m) / 2) / m,
        max_gen: *genset.iter().max().unwrap(),
        gen_set: genset,
        apery_set: aperyset,
    }
}

// ── GAP code generation ──────────────────────────────────────────────────────

/// Emit a GAP script (`NumericalSgps` package) that reconstructs each semigroup
/// and asserts all properties computed by this library, so the results can be
/// verified interactively in GAP.
#[must_use] 
pub fn to_gap(semigroups: &[Semigroup]) -> String {
    let mut out = String::new();

    out.push_str("# Generated by f3m — paste into GAP or run with 'gap <file>'\n");
    out.push_str("# Requires: LoadPackage(\"NumericalSgps\");\n\n");
    out.push_str("LoadPackage(\"NumericalSgps\");;\n\n");

    for (i, sg) in semigroups.iter().enumerate() {
        out.push_str(&gap_block(sg, i + 1));
    }
    out.push_str("Print(\"All assertions passed.\\n\");\n");
    out
}

/// Emit the GAP assertions for a single semigroup, using `ng{idx}` as the variable name.
#[must_use]
pub fn gap_block(sg: &Semigroup, idx: usize) -> String {
    use std::fmt::Write as _;
    let gens  = sg.gen_set  .iter().map(usize::to_string).collect::<Vec<_>>();
    let apery = sg.apery_set.iter().map(usize::to_string).collect::<Vec<_>>();
    let ((pf, t), _) = sg.pft();
    let pf_strs = pf.iter().map(usize::to_string).collect::<Vec<_>>();
    let sym = if sg.is_symmetric() { "true" } else { "false" };
    let mut out = String::new();
    writeln!(out, "# ── Semigroup {idx}: <{}> ──", gens.join(", ")).unwrap();
    writeln!(out, "ng{idx} := NumericalSemigroup({});", gens.join(",")).unwrap();
    writeln!(out, "Assert(0, Multiplicity(ng{idx}) = {});", sg.m).unwrap();
    writeln!(out, "Assert(0, FrobeniusNumber(ng{idx}) = {});", sg.f).unwrap();
    writeln!(out, "Assert(0, EmbeddingDimension(ng{idx}) = {});", sg.e).unwrap();
    writeln!(out, "Assert(0, GenusOfNumericalSemigroup(ng{idx}) = {});", sg.count_gap).unwrap();
    writeln!(out, "Assert(0, 1 + FrobeniusNumber(ng{idx}) - GenusOfNumericalSemigroup(ng{idx}) = {});", sg.count_set).unwrap();
    writeln!(out, "Assert(0, IsSymmetric(ng{idx}) = {sym});").unwrap();
    writeln!(out, "Assert(0, AperyList(ng{idx}, {}) = [{}]);", sg.m, apery.join(",")).unwrap();
    writeln!(out, "Assert(0, Set(PseudoFrobeniusOfNumericalSemigroup(ng{idx})) = Set([{}]));", pf_strs.join(",")).unwrap();
    writeln!(out, "Assert(0, TypeOfNumericalSemigroup(ng{idx}) = {t});").unwrap();
    out.push('\n');
    out
}
