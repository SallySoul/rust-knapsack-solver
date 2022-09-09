use crate::solver::problem::*;
use crate::solver::sol_tree::*;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
struct ItemEfficiency {
    index: usize,
    efficiency: f32,
}

fn efficiency_ordering(problem: &Problem) -> Vec<ItemEfficiency> {
    let mut item_efficiencies: Vec<ItemEfficiency> = problem
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if item.weight == 0 {
                panic!("Items with zero weight are not supported");
            }
            ItemEfficiency {
                index,
                efficiency: item.value as f32 / item.weight as f32,
            }
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

fn initial_bounds(
    problem: &Problem,
    item_efficiencies: &[ItemEfficiency],
) -> (BreakSolution, Vec<bool>) {
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
    let mut decision = vec![false; item_count];
    while i < item_count {
        let index = item_efficiencies[i].index;
        let item = &problem.items[index];
        if item.weight + weight_sum < problem.capacity {
            profit_sum += item.value;
            weight_sum += item.weight;
            decision[index] = true;
        } else {
            result.break_item = i;
            result.profit = profit_sum;
            result.weight = weight_sum;

            let remaining_weight = problem.capacity - weight_sum;
            let break_item_efficiency = item_efficiencies[i].efficiency;
            result.linear_profit =
                profit_sum + (remaining_weight as f32 * break_item_efficiency).ceil() as usize;
            break;
        }
        i += 1;
    }

    (result, decision)
}

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StateKey {
    c: usize,
    p: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StateValue {
    sol: SolCrumb,
}

pub struct Instance<'a> {
    best_sol_weight: usize,
    best_sol_item: usize,
    best_sol_level: usize,
    best_sol: SolCrumb,
    sol_level: usize,
    decision: Vec<bool>,
    item_order: Vec<usize>,
    item_efficiencies: Vec<ItemEfficiency>,
    break_solution: BreakSolution,
    problem: &'a Problem,
    s: usize,
    t: usize,
    lower_bound: usize,
    state_counter: usize,
    max_iter_state: usize,
}

impl<'a> Instance<'a> {
    fn new(problem: &Problem) -> Instance {
        let n = problem.items.len();
        let item_efficiencies = efficiency_ordering(problem);
        let (break_solution, decision) = initial_bounds(problem, &item_efficiencies);
        let lower_bound = break_solution.profit;
        let b = break_solution.break_item;
        let s = b;
        let t = b - 1;

        Instance {
            best_sol_weight: 0,
            best_sol_level: 0,
            best_sol_item: 0,
            best_sol: SolCrumb::new(0),
            sol_level: 0,
            decision,
            item_order: Vec::with_capacity(n),
            item_efficiencies,
            break_solution,
            problem,
            s,
            t,
            lower_bound,
            state_counter: 0,
            max_iter_state: 0,
        }
    }

    fn problem_capacity(&self) -> usize {
        self.problem.capacity
    }

    fn upper_bound(&self, s: &StateKey) -> usize {
        let n = self.item_count();
        if s.c <= self.problem_capacity() {
            // Under capacity
            if self.t < n - 1 {
                // Best we could do is linear add next t item
                let weight_remainder = (self.problem.capacity - s.c) as f32;
                let next_t_efficiency = self.item_efficiencies[self.t + 1].efficiency;
                s.p + (weight_remainder * next_t_efficiency).ceil() as usize
            } else {
                // No more items to add, we're done
                s.p
            }
        } else {
            // Over capacity
            if self.s > 0 {
                // Best we could do is linear remove next s item
                let weight_remainder = (s.c - self.problem.capacity) as f32;
                let next_s_efficiency = self.item_efficiencies[self.s - 1].efficiency;
                let linear_diff = (weight_remainder * next_s_efficiency).ceil() as usize;
                if linear_diff > s.p {
                    0
                } else {
                    s.p - linear_diff
                }
            } else {
                s.p
            }
        }
    }

    fn item_count(&self) -> usize {
        self.problem.items.len()
    }

    fn item(&self, ordered_index: usize) -> Item {
        let index = self.item_efficiencies[ordered_index].index;
        self.problem.items[index]
    }

    fn add_to_item_order(&mut self, ordered_index: usize) {
        let index = self.item_efficiencies[ordered_index].index;
        self.item_order.push(index);
    }

    fn add_item_t(
        &mut self,
        current_states: &HashMap<StateKey, StateValue>,
        next_states: &mut HashMap<StateKey, StateValue>,
    ) {
        self.add_to_item_order(self.t);
        let item = self.item(self.t);
        for (s, value) in current_states {
            // State if we add item
            if s.c + item.weight < 2 * self.problem_capacity() {
                let new_profit = s.p + item.value;
                let new_capacity = s.c + item.weight;
                let mut new_sol = value.sol;
                new_sol.add_decision(true);
                next_states.insert(
                    StateKey {
                        p: new_profit,
                        c: new_capacity,
                    },
                    StateValue { sol: new_sol },
                );
            }
            // Keep things as they are
            let mut old_v = *value;
            old_v.sol.add_decision(false);
            next_states.insert(*s, old_v);
        }
    }

    fn remove_item_s(
        &mut self,
        current_states: &HashMap<StateKey, StateValue>,
        next_states: &mut HashMap<StateKey, StateValue>,
    ) {
        self.add_to_item_order(self.s);
        let item = self.item(self.s);
        for (s, value) in current_states {
            // State if we add item
            if s.c >= item.weight {
                let new_profit = s.p - item.value;
                let new_capacity = s.c - item.weight;
                let mut new_sol = value.sol;
                new_sol.add_decision(true);
                next_states.insert(
                    StateKey {
                        p: new_profit,
                        c: new_capacity,
                    },
                    StateValue { sol: new_sol },
                );
            }

            // Keep things as they are
            let mut old_v = *value;
            old_v.sol.add_decision(false);
            next_states.insert(*s, old_v);
        }
    }

    fn reduce_states(
        &mut self,
        sol_tree: &mut SolTree,
        current_states: &mut HashMap<StateKey, StateValue>,
        next_states: &mut HashMap<StateKey, StateValue>,
    ) {
        self.state_counter += next_states.len();
        self.max_iter_state = self.max_iter_state.max(next_states.len());

        // Update lower bound
        for (s, value) in next_states.iter() {
            if s.c <= self.problem_capacity() && s.p > self.lower_bound {
                self.lower_bound = s.p;
                self.best_sol = value.sol;
                self.best_sol_level = self.sol_level + 1;
                self.best_sol_item = self.item_order.len() - 1;
                self.best_sol_weight = s.c;
                println!(
                    "New lower_bound, value: {}, item (visit order): {}",
                    self.lower_bound, self.best_sol_item
                );
            }
        }

        // Every reduce states call follows a decision for each state
        // We put this logic for icrementing the sol level here
        // Becuase hashsets have no mutable iterator
        // So we modify all the states as we filter them
        // With the added bonus of only saving history
        // for states that we're gonna keep
        self.sol_level += 1;
        if self.sol_level >= 64 {
            self.sol_level = 0;
            current_states.clear();
            current_states.extend(
                next_states
                    .drain()
                    .filter(|(s, _)| self.upper_bound(s) > self.lower_bound + 1)
                    .map(|(s, mut v)| {
                        sol_tree.fresh_crumb(&mut v.sol);
                        (s, v)
                    }),
            );
        } else {
            current_states.clear();
            current_states.extend(
                next_states
                    .drain()
                    .filter(|(s, _)| self.upper_bound(s) > self.lower_bound + 1),
            );
        }
    }

    fn backtrack_decision(&mut self, sol_tree: &mut SolTree) {
        // Return item order to where it was when best Solution
        // was last updated
        self.item_order.drain(self.best_sol_item + 1..);
        sol_tree.backtrack(
            self.best_sol,
            self.best_sol_level,
            &self.item_order,
            &mut self.decision,
        );
    }

    fn print_update(
        &self,
        i: usize,
        current_states: &HashMap<StateKey, StateValue>,
        sol_tree: &SolTree,
    ) {
        let n = self.item_count();
        // Scale gaps between iteration based on size of i
        let m = usize::pow(10, (i as f64).log10().floor() as u32);
        if i != 0 && (i < 10 || i % m == 0) {
            let core_width = (self.t - self.s) + 1;
            let core_percentage = 100.0 * (core_width as f32 / n as f32);
            println!(
                "Iteration i: {}, active states: {}, sol tree size: {}, core_size: %{:.4}",
                i,
                current_states.len(),
                sol_tree.len(),
                core_percentage,
            );
        }
    }

    fn solve(&mut self) {
        let mut current_states = HashMap::new();
        let mut next_states = HashMap::new();
        let n = self.item_count();
        let mut i = 0;
        current_states.insert(
            StateKey {
                p: self.break_solution.profit,
                c: self.break_solution.weight,
            },
            StateValue {
                sol: SolCrumb::new(0),
            },
        );

        let mut sol_tree = SolTree::new();
        while !current_states.is_empty() && i < n {
            self.print_update(i, &current_states, &sol_tree);

            if self.t < n - 1 {
                self.t += 1;
                self.add_item_t(&current_states, &mut next_states);
                self.reduce_states(&mut sol_tree, &mut current_states, &mut next_states);
                i += 1;
            }

            if self.s > 0 {
                self.s -= 1;
                self.remove_item_s(&current_states, &mut next_states);
                self.reduce_states(&mut sol_tree, &mut current_states, &mut next_states);
                i += 1;
            }
        }
        self.backtrack_decision(&mut sol_tree);
    }
}

pub fn solve(problem: &Problem) -> Result<Solution, Box<dyn std::error::Error>> {
    let mut instance = Instance::new(problem);
    instance.solve();
    Ok(Solution {
        decision: instance.decision,
        value: instance.lower_bound,
        weight: instance.best_sol_weight,
    })
}
