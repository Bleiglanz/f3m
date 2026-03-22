#![warn(clippy::pedantic)]
pub mod eva;
pub mod math;
pub mod jshelpers;

#[cfg(test)]
mod tests {
    use crate::math::{gcd, compute};

    #[test]
    fn test_ord_containment() {
        // <6,9,20> ⊂ <6,9,20,43> (adding the Frobenius number as a generator)
        let s1 = compute(&[6, 9, 20]);
        let s2 = compute(&[6, 9, 20, 43]);
        assert!(s1 < s2);
        assert!(s2 > s1);

        // Equal semigroups
        let s3 = compute(&[6, 9, 20]);
        assert_eq!(s1.partial_cmp(&s3), Some(std::cmp::Ordering::Equal));

        // <3,7> and <2,7> are incomparable: 2 ∉ <3,7> and 3 ∉ <2,7>
        let a = compute(&[3, 7]);
        let b = compute(&[2, 7]);
        assert_eq!(a.partial_cmp(&b), None);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 3), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(9, 0), 9);
    }

    /// Compute the semigroup from `gens` and assert every GAP-verified property.
    ///
    /// Parameters match the GAP assertions in gap/test.g:
    /// `e` = embedding dimension, `f` = Frobenius number, `m` = multiplicity,
    /// `g` = genus, `c` = count_set (= 1+f−g), `sym` = IsSymmetric,
    /// `apery` = AperyList w.r.t. m (indexed 0..m), `pf` = PseudoFrobenius set,
    /// `t` = type.
    #[allow(clippy::too_many_arguments)]
    fn check(
        gens:  &[usize],
        e:     usize,
        f:     usize,
        m:     usize,
        g:     usize,
        c:     usize,
        sym:   bool,
        apery: &[usize],
        pf:    &[usize],
        t:     usize,
    ) {
        let s = compute(gens);
        assert_eq!(s.e,         e,   "e     for {gens:?}");
        assert_eq!(s.f,         f,   "f     for {gens:?}");
        assert_eq!(s.m,         m,   "m     for {gens:?}");
        assert_eq!(s.count_gap, g,   "genus for {gens:?}");
        assert_eq!(s.count_set, c,   "c     for {gens:?}");
        assert_eq!(s.is_symmetric(), sym, "sym   for {gens:?}");
        assert_eq!(s.apery_set.as_slice(), apery, "apery for {gens:?}");
        let ((mut actual_pf, actual_t), _) = s.pseudo_and_special();
        actual_pf.sort_unstable();
        let mut expected_pf = pf.to_vec();
        expected_pf.sort_unstable();
        assert_eq!(actual_pf, expected_pf, "pf    for {gens:?}");
        assert_eq!(actual_t, t, "type  for {gens:?}");
    }

    // ── Examples from gap/test.g (first block, ng1–ng10) ─────────────────────

    #[test]
    fn test_6_9_20() {
        check(
            &[6, 9, 20],
            3, 43, 6, 22, 22, true,
            &[0, 49, 20, 9, 40, 29],
            &[43], 1,
        );
    }

    #[test]
    fn test_12_14_19_77() {
        check(
            &[12, 14, 19, 77],
            4, 65, 12, 38, 28, false,
            &[0, 61, 14, 75, 28, 77, 42, 19, 56, 33, 70, 47],
            &[58, 63, 65], 3,
        );
    }

    #[test]
    fn test_20_44_54_73_77_89_90() {
        check(
            &[20, 44, 54, 73, 77, 89, 90],
            7, 159, 20, 103, 57, false,
            &[0, 121, 142, 143, 44, 165, 146, 127, 88, 89, 90, 131, 132, 73, 54, 175, 176, 77, 98, 179],
            &[107, 122, 123, 126, 145, 155, 156, 159], 8,
        );
    }

    #[test]
    fn test_13_17_27() {
        check(
            &[13, 17, 27],
            3, 89, 13, 48, 42, false,
            &[0, 27, 54, 68, 17, 44, 71, 85, 34, 61, 88, 102, 51],
            &[75, 89], 2,
        );
    }

    #[test]
    fn test_20_28_30_46_47() {
        check(
            &[20, 28, 30, 46, 47],
            5, 129, 20, 71, 59, false,
            &[0, 121, 102, 103, 84, 105, 46, 47, 28, 149, 30, 131, 92, 93, 74, 75, 56, 77, 58, 139],
            &[85, 111, 119, 129], 4,
        );
    }

    #[test]
    fn test_18_40_49_51_78_93() {
        check(
            &[18, 40, 49, 51, 78, 93],
            6, 164, 18, 93, 72, false,
            &[0, 91, 182, 93, 40, 131, 78, 133, 80, 153, 100, 173, 102, 49, 140, 51, 142, 89],
            &[60, 122, 135, 155, 164], 5,
        );
    }

    #[test]
    fn test_46_47_54_62_66_69_70_85() {
        check(
            &[46, 47, 54, 62, 66, 69, 70, 85],
            8, 219, 46, 128, 92, false,
            &[0,47,94,141,188,189,190,237,54,101,148,195,242,151,198,245,
              62,109,156,203,66,113,160,69,70,117,164,211,120,167,214,123,
              124,171,218,265,128,175,222,85,132,179,226,135,136,183],
            &[143, 168, 176, 180, 191, 196, 199, 219], 8,
        );
    }

    #[test]
    fn test_11_15_17_38() {
        check(
            &[11, 15, 17, 38],
            4, 57, 11, 30, 28, false,
            &[0, 34, 68, 47, 15, 38, 17, 51, 30, 53, 32],
            &[36, 57], 2,
        );
    }

    #[test]
    fn test_19_24_27_32() {
        check(
            &[19, 24, 27, 32],
            4, 125, 19, 67, 59, false,
            &[0, 96, 59, 136, 80, 24, 120, 64, 27, 104, 48, 144, 88, 32, 128, 72, 54, 112, 56],
            &[35, 40, 109, 117, 125], 5,
        );
    }

    #[test]
    fn test_16_26_36_75_81_82() {
        check(
            &[16, 26, 36, 75, 81, 82],
            6, 167, 16, 86, 82, false,
            &[0, 81, 82, 147, 36, 101, 118, 183, 72, 137, 26, 75, 108, 157, 62, 111],
            &[65, 102, 121, 167], 4,
        );
    }

    // ── Examples from gap/test.g (second block) ───────────────────────────────

    #[test]
    fn test_12_30_32_38_40_41_symmetric() {
        check(
            &[12, 30, 32, 38, 40, 41],
            6, 99, 12, 50, 50, true,
            &[0, 73, 38, 111, 40, 41, 30, 79, 32, 81, 70, 71],
            &[99], 1,
        );
    }

    #[test]
    fn test_12_30_32_38_40_41_99() {
        check(
            &[12, 30, 32, 38, 40, 41, 99],
            7, 87, 12, 49, 39, false,
            &[0, 73, 38, 99, 40, 41, 30, 79, 32, 81, 70, 71],
            &[58, 59, 61, 67, 69, 87], 6,
        );
    }

    #[test]
    fn test_12_42_44_50_52_53_82_83_85_91_93() {
        check(
            &[12, 42, 44, 50, 52, 53, 82, 83, 85, 91, 93],
            11, 123, 12, 62, 62, true,
            &[0, 85, 50, 135, 52, 53, 42, 91, 44, 93, 82, 83],
            &[123], 1,
        );
    }

    #[test]
    fn test_12_102_104_110_112_113_142_143_145_151_153() {
        check(
            &[12, 102, 104, 110, 112, 113, 142, 143, 145, 151, 153],
            11, 243, 12, 122, 122, true,
            &[0, 145, 110, 255, 112, 113, 102, 151, 104, 153, 142, 143],
            &[243], 1,
        );
    }

    #[test]
    fn test_12_102_104_110_112_113_142_143_145_151_153_243() {
        check(
            &[12, 102, 104, 110, 112, 113, 142, 143, 145, 151, 153, 243],
            12, 231, 12, 121, 111, false,
            &[0, 145, 110, 243, 112, 113, 102, 151, 104, 153, 142, 143],
            &[90, 92, 98, 100, 101, 130, 131, 133, 139, 141, 231], 11,
        );
    }
}
