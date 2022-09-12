use crate::solver::problem::*;

struct RatioItem {
    index: usize,
    ratio: f32,
}

pub fn solve(problem: &Problem) -> Solution {
    let mut ratios: Vec<RatioItem> = problem
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| RatioItem {
            index,
            ratio: item.value as f32 / item.weight as f32,
        })
        .collect();

    // We want Highest ratio to lowest
    // Hence b cmp a
    ratios.sort_unstable_by(|a, b| b.ratio.partial_cmp(&a.ratio).unwrap());

    let mut decision = vec![false; problem.items.len()];
    let mut weight_sum = 0;
    let mut value_sum = 0;
    for r in &ratios {
        let item = &problem.items[r.index];
        if weight_sum + item.weight < problem.capacity {
            weight_sum += item.weight;
            value_sum += item.value;
            decision[r.index] = true;
        }
    }

    Solution {
        decision,
        value: value_sum,
        weight: weight_sum,
    }
}
