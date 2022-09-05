use crate::solver::problem::*;

use std::hash::{Hash, Hasher};

struct ItemEfficiency {
    index: usize,
    efficiency: f32,
}

fn efficiency_ordering(problem: &Problem) -> Vec<ItemEfficiency> {
    let mut item_efficiencies: Vec<ItemEfficiency> = problem
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| ItemEfficiency {
            index,
            efficiency: item.value as f32 / item.weight as f32,
        })
        .collect();
    // We want Highest ratio to lowest
    // Hence b cmp a
    item_efficiencies.sort_unstable_by(|a, b| b.efficiency.partial_cmp(&a.efficiency).unwrap());

    item_efficiencies
}

struct BreakSolution {
    break_item: usize,
    profit: usize,
    weight: usize,
    linear_profit: usize,
}

fn initial_bounds(problem: &Problem, item_efficiencies: &Vec<ItemEfficiency>) -> BreakSolution {
    let item_count = problem.items.len();
    let mut result = BreakSolution {
        break_item: 0,
        profit: 0,
        weight: 0,
        linear_profit: 0,
    };
    let mut profit_sum = 0;
    let mut weight_sum = 0;
    let mut i = 0;
    while i < item_count {
        let index = item_efficiencies[i].index;
        let item = &problem.items[index];
        if item.weight + weight_sum < problem.capacity {
            profit_sum += item.value;
            weight_sum += item.weight;
        } else {
            result.break_item = i;
            result.profit = profit_sum;
            result.weight = weight_sum;

            let remaining_weight = problem.capacity - weight_sum;
            let break_item_efficiency = item_efficiencies[i].efficiency;
            result.linear_profit =
                profit_sum + (remaining_weight as f32 * break_item_efficiency).ceil() as usize;
            return result;
        }
        i += 1;
    }

    // Should never hit this?
    panic!("All items in solutions?");
    /*
    result.break_item = item_count - 1;
    result.initial_lb = profit_sum;
    result.initial_ub = profit_sum;
    return result
    */
}

#[derive(Hash)]
struct StateKey {
    s: usize,
    t: usize,
    capacity: i32,
}

impl StateKey {
    fn new(s: usize, t: usize, capacity: i32) -> StateKey {
        StateKey { s, t, capacity }
    }
}

struct StateValue {
    profit: i32,
    sol: usize,
}

struct StateMap<'a> {
    states: std::collections::HashMap<StateKey, StateValue>,
    item_efficiencies: Vec<ItemEfficiency>,
    break_solution: BreakSolution,
    problem: &'a Problem,
}

impl<'a> StateMap<'a> {
    fn break_item(&self) -> usize {
        self.break_solution.break_item
    }

    fn break_weight(&self) -> i32 {
        self.break_solution.weight as i32
    }

    fn break_profit(&self) -> i32 {
        self.break_solution.profit as i32
    }

    fn over_capacity(&self) -> i32 {
        // From paper? ehhhh
        self.problem.capacity as i32 * 2
    }

    fn ordered_item(&self, ordered_index: usize) -> &Item {
        let item_index = self.item_efficiencies[ordered_index].index;
        &self.problem.items[item_index]
    }

    fn new(problem: &Problem) -> StateMap {
        let item_efficiencies = efficiency_ordering(problem);
        let break_solution = initial_bounds(problem, &item_efficiencies);

        StateMap {
            states: std::collections::HashMap::new(),
            item_efficiencies,
            break_solution,
            problem,
        }
    }

    fn get(&mut self, key: StateKey) -> StateValue {
        println!("Get State s: {}, t: {}, c: {}", key.s, key.t, key.capacity);
        if key.s == self.break_item() && key.t == self.break_item() - 1 {
            if key.capacity < self.break_weight() {
                println!("  base case, less than break");
                return StateValue {
                    profit: std::i32::MIN,
                    sol: 0,
                };
            } else {
                println!("  base case, greater than break");
                return StateValue {
                    profit: self.break_profit(),
                    sol: 0,
                };
            }
        }

        let mut new_profit = std::i32::MIN;

        assert!(key.t >= self.break_item());
        assert!(key.s < self.break_item());

        let item_s_index = self.item_efficiencies[key.s].index;
        let item_t_index = self.item_efficiencies[key.t].index;
        let item_s = &self.problem.items[item_s_index];
        let item_t = &self.problem.items[item_t_index];

        let sv_1 = self
            .get(StateKey::new(key.s, key.t - 1, key.capacity))
            .profit;
        new_profit = new_profit.max(sv_1);

        if key.capacity - (item_t.weight as i32) >= 0 {
            let sv_2 = self
                .get(StateKey::new(
                    key.s,
                    key.t - 1,
                    key.capacity - item_t.weight as i32,
                ))
                .profit
                + item_t.value as i32;
            new_profit = new_profit.max(sv_2);
        }

        let sv_3 = self
            .get(StateKey::new(key.s + 1, key.t, key.capacity))
            .profit;
        new_profit = new_profit.max(sv_3);

        if key.capacity + (item_s.weight as i32) < self.over_capacity() {
            let sv_4 = self
                .get(StateKey::new(
                    key.s + 1,
                    key.t,
                    key.capacity + item_s.weight as i32,
                ))
                .profit
                - item_s.value as i32;
            new_profit = new_profit.max(sv_4);
        }

        println!(
            "  new profit: {}, s: {}, t: {}, c: {}",
            new_profit, key.s, key.t, key.capacity
        );

        // So... Lower bound is
        //   - Starts with break solution
        //   - Improve as we find states with higher profit under capacity limit
        // We can compute upper bound with the whole
        // weigth difference times next item thing

        StateValue { profit: 0, sol: 0 }
    }
}

pub fn solve(problem: &Problem) -> Result<Solution, Box<dyn std::error::Error>> {
    let mut state_map = StateMap::new(problem);
    let break_item = state_map.break_item();
    let w = state_map.ordered_item(break_item + 1).weight as i32;
    let bs = state_map.break_profit() as i32;
    println!("Break Item: {}", break_item);
    state_map.get(StateKey::new(break_item - 1, break_item + 1, bs + w));

    Ok(Solution {
        decision: vec![false; 0],
        value: 0,
    })
}
