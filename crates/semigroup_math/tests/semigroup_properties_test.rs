use semigroup_math::math::Semigroup;

//for properties that do not depend on the values for prop in the csv
//internal data is used, everything from the semigroup itself (consistency)
pub fn basic(s: &Semigroup) {
    //println!("test {s:?}");
    if s.m > 1 {
        assert_eq!(s.diag(s.mu), s.r)
    };
    let gens = s.gen_set.clone();
    // todo 82 / 85
    // rl < t whenever t > 0: every gap counted by rl is pseudo-Frobenius and
    // strictly below f, while f itself is always pseudo-Frobenius and never
    // counted by rl. So PF ⊇ {rl-counted gaps} ⊔ {f}, hence rl + 1 ≤ t.
    if s.t > 0 {
        assert!(s.rl < s.t, "rl = {}, t = {} ", s.rl, s.t);
    } else {
        assert_eq!(s.rl, 0);
    }
    assert!(s.t <= s.r + 1);
    assert!(s.es <= s.e);
    assert_eq!(s.f + s.m, s.apery_set[s.mu], "max apery");
    for i in 0..s.m {
        assert_eq!(s.diag(i) + s.apery_set[i], s.f + s.m + s.r, "diag {i}");
    }
    assert_eq!(s.diag(s.mu), s.r, "{gens:?}");
    assert_eq!(s.sigma + s.r, s.g, "g ist sigma + r");
    assert_eq!(s.f + s.r + 2, 2 * s.g + 1, "f+r+2=2g+1");
    assert_eq!(
        s.ae,
        *s.gen_set.iter().max().unwrap(),
        "maximum for {gens:?}"
    );
    assert!(s.rho <= s.tau);
    assert!(s.apn[0] >= 2);
    if s.ae == (s.f + s.m) {
        assert_eq!(s.apn[0], 2);
    }
    let apn_sum: usize = s.apn.iter().sum();
    assert_eq!(apn_sum, s.m);
    assert_eq!(s.apn.len(), s.tau + 1);
    let apn_rsum: usize = (0..s.tau + 1).map(|x| s.apn[x] * x).sum();
    assert_eq!(apn_rsum, s.r);
    //let round = s.descent_flip().ascent_flip();
    //assert_eq!([s.g,s.sigma,s.r],[round.g,round.sigma,round.r],"{gens:?}")
    assert!((0..s.m).all(|i| s.r_i(i) == s.rvec[i]));
    let add_f_to_semigroup = s.toggle(s.f);
    if s.mu == add_f_to_semigroup.mu && s.m == add_f_to_semigroup.m && s.m > 2 {
        assert_eq!(s.tau + 1, add_f_to_semigroup.tau);
        for i in 1..s.tau + 1 {
            if i < s.apn.len() {
                assert_eq!(
                    s.apn[i],
                    add_f_to_semigroup.apn[i + 1],
                    "{:?} vs {:?} for {gens:?} at i={i}",
                    s.apn,
                    add_f_to_semigroup.apn
                );
            }
        }
    }
    // the sum of all apns must equal m (because they slice m)
    let apnsum = s.apn.iter().sum();
    assert_eq!(s.m, apnsum);

    //almost symmetrical
    if s.r + 1 == s.t && s.m>1 {
        assert_eq!(2 * s.g, s.f + s.t, "2g=f+t {gens:?}");
        assert!(s.tau <= 1);
        assert!(s.r <= s.m - 2);
        assert!(s.is_almost_symmetric);
        if s.tau >= 1 {
            assert_eq!(s.apn[0] + s.apn[1], s.m, "apn only in 0,1 for {gens:?}");
        }
        if s.v_in_s() {
            let sprime = s.toggle(s.f);
            assert_eq!(sprime.f + s.m, s.f);
            assert_eq!(sprime.m, s.m);
            assert_eq!(sprime.g + 1, s.g);
            assert_eq!(sprime.r + 2, s.r + s.m);
            (1..s.m).all(|i| s.rvec[i] + 1 == sprime.rvec[i]);
            if s.tau == 1 {
                assert_eq!(sprime.tau, 2)
            };
            if s.rho < s.tau {
                assert_eq!(sprime.rho, 1, "sprime tau {gens:?}")
            };
            let spminusf: Vec<usize> = s.pf_set.iter().copied().filter(|&x| x < s.f).collect();
            assert!(spminusf.iter().all(|x| sprime.pf_set.contains(x)));
            assert_eq!(sprime.gen_set[sprime.e-1],sprime.f+sprime.m);
            let generator_deltas: Vec<usize> = sprime.gen_set[0..sprime.e - 1]
                .iter()
                .filter(|&a| s.pf_set.iter().all(|&l| l == s.f  || *a < l ||
                    !sprime.element(a-l)
                ))
                .map(|a| sprime.max_gen - a)
                .collect();
            assert!(
                sprime
                    .pf_set
                    .iter()
                    .all(|x| { s.pf_set.contains(x) || generator_deltas.contains(x) }),
                "PF of S' for {gens:?}"
            );
            let mut newpf:Vec<usize> = s.pf_set.iter().copied().filter(|&x| x < s.f).collect();
            newpf.extend(generator_deltas.clone());
            newpf.dedup();
            assert!(newpf.iter().all(
                |x|
                    sprime.pf_set.contains(x)),
                    "pf S' for {:?} : \ncandidate: {newpf:?}\nshould be:  {:?}\nold pf (S): {:?} old r={}\ngenerator deltas {:?} "
                    , sprime.gen_set,                                 sprime.pf_set, s.pf_set, s.r, generator_deltas);


        }
    }
}
