use crate::solver::problem::*;
use crate::solver::sol_tree::*;
use std::collections::HashSet;

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
    item_efficiencies: &Vec<ItemEfficiency>,
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
    let mut decision = Vec::with_capacity(item_count);
    while i < item_count {
        let index = item_efficiencies[i].index;
        let item = &problem.items[index];
        if item.weight + weight_sum < problem.capacity {
            profit_sum += item.value;
            weight_sum += item.weight;
            decision.push(true);
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

    while i < item_count {
        decision.push(false);
        i += 1;
    }

    // TODO debug assert?
    assert_eq!(decision.len(), item_count);

    (result, decision)
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct State {
    c: usize,
    p: usize,
    sol: SolCrumb,
}

pub struct Instance<'a> {
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

    fn upper_bound(&self, s: &State) -> usize {
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

    fn add_item_t(&mut self, current_states: &HashSet<State>, next_states: &mut HashSet<State>) {
        //println!("  add_item {}", self.t);
        self.add_to_item_order(self.t);
        let item = self.item(self.t);
        for s in current_states {
            // State if we add item
            if s.c + item.weight < 2 * self.problem_capacity() {
                let new_profit = s.p + item.value;
                let new_capacity = s.c + item.weight;
                let mut new_sol = s.sol;
                new_sol.add_decision(true);
                next_states.insert(State {
                    p: new_profit,
                    c: new_capacity,
                    sol: new_sol,
                });
            }
            // Keep things as they are
            let mut old_s = *s;
            old_s.sol.add_decision(false);
            next_states.insert(old_s);
        }
    }

    fn remove_item_s(&mut self, current_states: &HashSet<State>, next_states: &mut HashSet<State>) {
        //println!("  remove_item {}", self.s);
        self.add_to_item_order(self.s);
        let item = self.item(self.s);
        for s in current_states {
            // State if we add item
            if s.c >= item.weight {
                let new_profit = s.p - item.value;
                let new_capacity = s.c - item.weight;
                let mut new_sol = s.sol;
                new_sol.add_decision(true);
                next_states.insert(State {
                    p: new_profit,
                    c: new_capacity,
                    sol: new_sol,
                });
            }

            // Keep things as they are
            let mut old_s = *s;
            old_s.sol.add_decision(false);
            next_states.insert(old_s);
        }
    }

    fn reduce_states(
        &mut self,
        sol_tree: &mut SolTree,
        current_states: &mut HashSet<State>,
        next_states: &mut HashSet<State>,
    ) {
        //println!("  reduce_states");

        self.state_counter += next_states.len();
        self.max_iter_state = self.max_iter_state.max(next_states.len());

        // Update lower bound
        for s in next_states.iter() {
            if s.c <= self.problem_capacity() && s.p > self.lower_bound {
                self.lower_bound = s.p;
                self.best_sol = s.sol;
                //println!("    found new lower_bound: {}", self.lower_bound);
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
                    .filter(|s| self.upper_bound(s) > self.lower_bound)
                    .map(|mut s| {
                        sol_tree.fresh_crumb(&mut s.sol);
                        s
                    }),
            );
        } else {
            current_states.clear();
            current_states.extend(
                next_states
                    .drain()
                    .filter(|s| self.upper_bound(s) > self.lower_bound),
            );
        }

        /*
        let diff = state_count - current_states.len();
        println!(
            "  reduced {} states, {} -> {}",
            diff,
            state_count,
            current_states.len()
        );
        */
    }

    fn backtrack_decision(&mut self, sol_tree: &mut SolTree) {
        // Since reduce states increments all kept states
        sol_tree.backtrack(
            self.best_sol,
            self.sol_level,
            &self.item_order,
            &mut self.decision,
        );
    }

    fn solve(&mut self) {
        let mut current_states = HashSet::new();
        let mut next_states = HashSet::new();
        let n = self.item_count();
        let mut i = 0;
        current_states.insert(State {
            p: self.break_solution.profit,
            c: self.break_solution.weight,
            sol: SolCrumb::new(0),
        });

        let mut sol_tree = SolTree::new();
        while !current_states.is_empty() && i < n {
            if i % 100 == 0 {
                println!(
                    "Iteration i: {}, active states: {}, sol tree size: {}",
                    i,
                    current_states.len(),
                    sol_tree.len(),
                );
            }

            let n = self.item_count();
            if self.t < n - 1 {
                self.t += 1;
                self.add_item_t(&current_states, &mut next_states);
                self.reduce_states(&mut sol_tree, &mut current_states, &mut next_states);
            }

            if self.s > 0 {
                self.s -= 1;
                self.remove_item_s(&current_states, &mut next_states);
                self.reduce_states(&mut sol_tree, &mut current_states, &mut next_states);
            }

            i += 1;
        }
        self.backtrack_decision(&mut sol_tree);
    }
}

pub fn solve(problem: &Problem) -> Result<Solution, Box<dyn std::error::Error>> {
    let mut instance = Instance::new(problem);
    instance.solve();

    println!(
        "lb: {}, sc: {}, mc: {}",
        instance.lower_bound, instance.state_counter, instance.max_iter_state
    );

    Ok(Solution {
        decision: instance.decision,
        value: instance.lower_bound,
    })
}
