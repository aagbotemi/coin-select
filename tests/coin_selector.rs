mod common;
use bdk_coin_select::{Candidate, CoinSelector, Target, TargetFee, TargetOutputs};
#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use proptest::{prelude::*, proptest, test_runner::*};

fn test_wv(mut rng: impl RngCore) -> impl Iterator<Item = Candidate> {
    core::iter::repeat_with(move || {
        let value = rng.random_range(0..1_000);
        let mut candidate = Candidate {
            value,
            weight: 100,
            input_count: rng.random_range(1..2),
            is_segwit: rng.random_bool(0.5),
        };
        // HACK: set is_segwit = true for all these tests because you can't actually lower bound
        // things easily with how segwit inputs interfere with their weights. We can't modify the
        // above since that would change what we pull from rng.
        candidate.is_segwit = true;
        candidate
    })
}

proptest! {
    #[test]
    fn feasibility_check_never_false_negative(
        n in 1usize..12,                 // small: 2^12 subsets brute-forceable
        target_value in 0u64..5_000,
        max_weight in 500u64..3_000,
    ) {
        let mut rng = TestRng::deterministic_rng(RngAlgorithm::ChaCha);
        let candidates: Vec<Candidate> = test_wv(&mut rng).take(n).collect();
        let target = Target {
            fee: TargetFee::default(),
            outputs: TargetOutputs { value_sum: target_value, weight_sum: 0, n_outputs: 1 },
            max_weight: Some(max_weight),
        };
        let cs = CoinSelector::new(&candidates);

        // ground truth: brute-force all subsets
        let truth = (0..(1u32 << n)).any(|mask| {
            let mut t = cs.clone();
            for i in 0..n { if mask & (1 << i) != 0 { t.select(i); } }
            t.is_target_met(target)
        });

        // the check may say "possible" when it isn't (false positive OK),
        // but must NEVER say "impossible" when truth says possible:
        if truth { prop_assert!(cs.is_selection_possible(target)); }
    }
}
