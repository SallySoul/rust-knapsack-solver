use crate::converter::*;
use crate::solver::problem::*;
use crate::solver::sol_tree::*;
use std::mem::size_of;

#[derive(Debug)]
struct ItemEfficiency {
    index: usize,
    efficiency: f32,
}

fn efficiency_ordering(problem: &Problem) -> (Vec<ItemEfficiency>, Vec<bool>, usize) {
    let problem_item_count = problem.items.len();
    let mut decision = vec![false; problem_item_count];
    let mut base_value = 0;
    let mut item_efficiencies: Vec<ItemEfficiency> = problem
        .items
        .iter()
        .enumerate()
        // Variable reduction
        // Remove items that are larger than the capacity
        .filter(|(_, item)| item.weight <= problem.capacity)
        // Variable reduction
        // Remove items that are zero weight
        .filter(|(index, item)| {
            let check = item.weight != 0;
            if !check {
                decision[*index] = true;
                base_value += item.value;
            }
            check
        })
        .map(|(index, item)| {
            if item.weight == 0 {
                panic!("Items with zero weight should have been removed");
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

    (item_efficiencies, decision, base_value)
}

struct BreakSolution {
    break_item: usize,
    profit: usize,
    weight: usize,
    linear_profit: usize,
}

/// Calculate the break solution and populate the initial decision vector
/// Any decisions we make are modifications to the break decision vector
fn break_solution(
    problem: &Problem,
    item_efficiencies: &[ItemEfficiency],
    decision: &mut [bool],
) -> BreakSolution {
    // This is the number of items in the reduced problem
    let item_count = item_efficiencies.len();
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

    // Edge case, all items in solution
    if i == item_count {
        result.break_item = item_count;
        result.profit = profit_sum;
        result.weight = weight_sum;
    }

    result
}

/// Utility function for add and remove item funtions
/// only use when next_states is known to not be empty
fn last_profit(next_state: &[State]) -> usize {
    next_state.last().unwrap().p
}

/// Utility function for add and remove item funtions
/// only use when next_states is known to not be empty
fn last_weight(next_state: &[State]) -> usize {
    next_state.last().unwrap().w
}

#[derive(Debug, Copy, Clone)]
pub struct State {
    w: usize,
    p: usize,
    sol: SolCrumb,
}

/// Utility type
/// Wanted named arguments for lower_bound function
/// Easy to mix up four usizes
struct UBCheck {
    next_s: Option<usize>, // None if next s would be negative
    next_t: usize,
    new_weight: usize,
    new_profit: usize,
}

/// Most of the state needed for MinKnap function
/// Notably, state buffers are not included to simplify
/// ownership situation
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
    max_state_weight: usize,
    last_log_update: std::time::Instant,
    bytes_used: usize,
    states_explored: usize,
    base_value: usize,
}

impl<'a> Instance<'a> {
    fn new(problem: &Problem) -> Instance {
        let (item_efficiencies, mut decision, base_value) = efficiency_ordering(problem);
        let n = item_efficiencies.len();
        let break_solution = break_solution(problem, &item_efficiencies, &mut decision);
        let lower_bound = break_solution.profit;
        let b = break_solution.break_item;
        let s = b;
        let t = b - 1;
        let max_state_weight = problem.capacity + break_solution.weight;
        let bytes_used = size_of::<Instance>()
            + size_of::<Problem>()
            + (decision.capacity() * size_of::<bool>())
            + (item_efficiencies.capacity() * size_of::<ItemEfficiency>())
            + (problem.items.capacity() * size_of::<Item>());

        Instance {
            best_sol_weight: break_solution.weight,
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
            max_state_weight,
            last_log_update: std::time::Instant::now(),
            bytes_used,
            states_explored: 0,
            base_value,
        }
    }

    fn item_count(&self) -> usize {
        self.item_efficiencies.len()
    }

    fn problem_capacity(&self) -> usize {
        self.problem.capacity
    }

    fn item(&self, ordered_index: usize) -> Item {
        let index = self.item_efficiencies[ordered_index].index;
        self.problem.items[index]
    }

    // We calculate the upper bound by relaxing the integer
    // decision constraint. The best we can do with linear decisions
    // is to add or remove some ammount of the next most efficient / in-efficient item
    fn upper_bound(&self, u: UBCheck) -> usize {
        let n = self.item_count();
        if u.new_weight <= self.problem_capacity() {
            // Under capacity
            if u.next_t < n {
                // Best we could do is linear add next t item
                let weight_remainder = (self.problem.capacity - u.new_weight) as f32;
                let next_t_efficiency = self.item_efficiencies[u.next_t].efficiency;
                u.new_profit + (weight_remainder * next_t_efficiency).ceil() as usize
            } else {
                // No more items to add, we're done
                u.new_profit
            }
        } else {
            // Over capacity
            if u.next_s.is_some() {
                // Best we could do is linear remove next s item
                let weight_remainder = (u.new_weight - self.problem.capacity) as f32;
                let next_s_efficiency = self.item_efficiencies[u.next_s.unwrap()].efficiency;
                let linear_diff = (weight_remainder * next_s_efficiency).ceil() as usize;
                if linear_diff > u.new_profit {
                    0
                } else {
                    u.new_profit - linear_diff
                }
            } else {
                // No more items to remove, we're done
                u.new_profit
            }
        }
    }

    /// Use item_order buffer to track items as we add them to the core
    /// We are then able to use this buffer while backtracking the solution
    /// to update our decision vector
    fn add_to_item_order(&mut self, ordered_index: usize) {
        let index = self.item_efficiencies[ordered_index].index;
        self.item_order.push(index);
    }

    /// When changing a decision from the break solution, we need to check
    /// for a new lower bound
    fn check_for_new_lower_bound(&mut self, s: &State) {
        if s.w <= self.problem_capacity() && s.p > self.lower_bound {
            self.lower_bound = s.p;
            self.best_sol = s.sol;
            self.best_sol_level = self.sol_level + 1;
            self.best_sol_item = self.item_order.len() - 1;
            self.best_sol_weight = s.w;
            /*
            println!(
                "New lower bound found, {}, weight: {}, sol: {:064b}",
                self.lower_bound, self.best_sol_weight, self.best_sol.recent
            );
            */
        }
    }

    /// Trying adding item at sorted index self.t to the core
    fn add_item_t(&mut self, current_states: &[State], next_states: &mut Vec<State>) {
        // For every state, we need to try both adding and not adding the item
        // However, we also need to maintain profit and weight ordering of states
        // Such that duplicates and dominated states can be discarded
        //
        // To do so we iterate through all states with two indices
        // change_index is the next state we will try chaning
        // keep_index is the next state we are not changing
        // With some checks, we can be sure that we build an ordered next_states buffer
        // such that
        // value_i < value_{i + 1} weight_i < weight_{i + 1}
        // If we encounter a state with less profit at a greater weight
        // than the current next state, it is "dominated" and can be discarded
        //
        // This function is similiar to `void add_item` from
        // https://github.com/fontanf/knapsacksolver/blob/master/knapsacksolver/algorithms/minknap.cpp
        // An invaluable reference for figuring it out
        //
        // Also worth noting that an earlier version of this solver used a HashMap instead of the
        // sorted buffer. While still functional, hashing was a significant portion of the runtime
        // and an order of magnitude more states needed to be explored
        self.add_to_item_order(self.t);
        let item = self.item(self.t);
        let state_count = current_states.len();
        let mut change_index = 0;
        let mut keep_index = 0;
        while change_index != state_count || keep_index != state_count {
            // Check whether we do the next change_index
            // If there are no more keep_index or the next keep state would be greater than the
            // next change state
            if keep_index >= state_count
                || current_states[keep_index].w > current_states[change_index].w + item.weight
            {
                let change_state = current_states[change_index];
                let change_weight = change_state.w + item.weight;

                // Based on the paper, we only need to consider states with weight up to 2 *
                // problem capacity.
                // If more aggressive variable reduction were employed, this could be significantly
                // reduced.
                if change_weight > self.max_state_weight {
                    change_index += 1;
                    continue;
                }

                // This would be a dominated state, discard
                let change_profit = change_state.p + item.value;
                if !next_states.is_empty() && change_profit <= last_profit(next_states) {
                    change_index += 1;
                    continue;
                }

                // Ensure this state passes bounds check
                let upper_bound = self.upper_bound(UBCheck {
                    next_s: Some(self.s),
                    next_t: self.t + 1,
                    new_profit: change_profit,
                    new_weight: change_weight,
                });
                if upper_bound <= self.lower_bound {
                    change_index += 1;
                    continue;
                }

                let mut change_sol = change_state.sol;
                change_sol.add_decision(true);
                let new_state = State {
                    p: change_profit,
                    w: change_weight,
                    sol: change_sol,
                };

                // Only changed states can create a new lower bound
                self.check_for_new_lower_bound(&new_state);

                // If this state dominates the current last state, overwrite,
                // otherwise add the new state
                if !next_states.is_empty() && change_weight == last_weight(next_states) {
                    let last_index = next_states.len() - 1;
                    next_states[last_index] = new_state;
                } else {
                    next_states.push(new_state);
                }
                change_index += 1;
            } else {
                let keep_state = current_states[keep_index];
                debug_assert!(keep_index < state_count);
                debug_assert!(keep_state.w <= self.max_state_weight);
                if keep_state.w > self.max_state_weight {
                    keep_index += 1;
                    continue;
                }

                if !next_states.is_empty() && keep_state.p <= last_profit(next_states) {
                    //debug_assert!(keep_state.w <= last_weight(next_states));
                    keep_index += 1;
                    continue;
                }

                let upper_bound = self.upper_bound(UBCheck {
                    next_s: Some(self.s),
                    next_t: self.t + 1,
                    new_profit: keep_state.p,
                    new_weight: keep_state.w,
                });
                if upper_bound <= self.lower_bound {
                    keep_index += 1;
                    continue;
                }

                let mut new_state = keep_state;
                new_state.sol.add_decision(false);
                if !next_states.is_empty() && keep_state.w == last_weight(next_states) {
                    let last_index = next_states.len() - 1;
                    next_states[last_index] = new_state;
                } else {
                    next_states.push(new_state);
                }
                keep_index += 1;
            }
        }
    }

    /// Trying removing item at sorted index self.s to the core
    fn remove_item_s(&mut self, current_states: &[State], next_states: &mut Vec<State>) {
        // Similiar to add_item, see comments there
        self.add_to_item_order(self.s);
        let item = self.item(self.s);
        let state_count = current_states.len();
        let mut change_index = 0;
        let mut keep_index = 0;
        while change_index != state_count || keep_index != state_count {
            if change_index >= state_count
                || current_states[keep_index].w <= current_states[change_index].w - item.weight
            {
                let keep_state = current_states[keep_index];
                debug_assert!(keep_index < state_count);

                debug_assert!(keep_state.w < self.max_state_weight);
                if keep_state.w > self.max_state_weight {
                    keep_index += 1;
                    continue;
                }

                if !next_states.is_empty() && keep_state.p <= last_profit(next_states) {
                    keep_index += 1;
                    continue;
                }

                let upper_bound = self.upper_bound(UBCheck {
                    next_s: if self.s > 0 { Some(self.s - 1) } else { None },
                    next_t: self.t,
                    new_profit: keep_state.p,
                    new_weight: keep_state.w,
                });
                if upper_bound <= self.lower_bound {
                    keep_index += 1;
                    continue;
                }

                let mut new_state = keep_state;
                new_state.sol.add_decision(false);
                if !next_states.is_empty() && keep_state.w == last_weight(next_states) {
                    let last_index = next_states.len() - 1;
                    next_states[last_index] = new_state;
                } else {
                    next_states.push(new_state);
                }
                keep_index += 1;
            } else {
                let change_state = current_states[change_index];
                let change_weight = change_state.w - item.weight;

                debug_assert!(change_state.p >= item.value);
                let change_profit = change_state.p - item.value;
                if !next_states.is_empty() && change_profit <= last_profit(next_states) {
                    change_index += 1;
                    continue;
                }

                let upper_bound = self.upper_bound(UBCheck {
                    next_s: if self.s > 0 { Some(self.s - 1) } else { None },

                    next_t: self.t,
                    new_profit: change_profit,
                    new_weight: change_weight,
                });
                if upper_bound <= self.lower_bound {
                    change_index += 1;
                    continue;
                }

                let mut change_sol = change_state.sol;
                change_sol.add_decision(true);
                let new_state = State {
                    p: change_profit,
                    w: change_weight,
                    sol: change_sol,
                };
                self.check_for_new_lower_bound(&new_state);
                if !next_states.is_empty() && change_weight == last_weight(next_states) {
                    let last_index = next_states.len() - 1;
                    next_states[last_index] = new_state;
                } else {
                    next_states.push(new_state);
                }
                change_index += 1;
            }
        }
    }

    fn swap_state_buffers(
        &mut self,
        current_states: &mut Vec<State>,
        next_states: &mut Vec<State>,
    ) {
        current_states.clear();
        std::mem::swap(current_states, next_states);
        self.states_explored += current_states.len();
    }

    fn backtrack_decision(&mut self, sol_tree: &SolTree) {
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

    fn bytes_estimate(
        &mut self,
        current_states: &Vec<State>,
        next_states: &Vec<State>,
        sol_tree: &SolTree,
    ) -> usize {
        let state_bytes = (current_states.capacity() + next_states.capacity()) * size_of::<State>();
        let sol_tree_bytes = sol_tree.bytes_used();
        let item_order_bytes = self.item_order.capacity() * size_of::<usize>();
        self.bytes_used + state_bytes + sol_tree_bytes + item_order_bytes
    }

    fn print_update(
        &mut self,
        i: usize,
        current_states: &Vec<State>,
        next_states: &Vec<State>,
        sol_tree: &SolTree,
    ) {
        let n = self.item_count();
        let elapsed_time = self.last_log_update.elapsed().as_millis();
        if i != 0 && ((i < 10 || (i < 100 && i % 10 == 0)) || elapsed_time > 1500) {
            self.last_log_update = std::time::Instant::now();
            let core_width = (self.t - self.s) + 1;
            let core_percentage = 100.0 * (core_width as f32 / n as f32);
            let bytes_estimate = self.bytes_estimate(current_states, next_states, sol_tree);
            let hr_bytes = human_readable_bytes(bytes_estimate);
            println!(
                "iteration i: {}, active_states: {}, core_size: %{:.4}, mem_used: {} ({} bytes)",
                i,
                current_states.len(),
                core_percentage,
                hr_bytes,
                bytes_estimate,
            );
        }
    }

    fn print_final_update(
        &mut self,
        i: usize,
        current_states: &Vec<State>,
        next_states: &Vec<State>,
        sol_tree: &SolTree,
    ) {
        let n = self.item_count();
        let core_width = (self.t - self.s) + 1;
        let core_percentage = 100.0 * (core_width as f32 / n as f32);
        let bytes_estimate = self.bytes_estimate(current_states, next_states, sol_tree);
        let hr_bytes = human_readable_bytes(bytes_estimate);
        println!(
            "final i: {}, states_explored: {}, core_size: %{:.4}, mem_used: {} ({} bytes)",
            i, self.states_explored, core_percentage, hr_bytes, bytes_estimate,
        );
    }

    fn backup_solution_history(&mut self, sol_tree: &mut SolTree, current_states: &mut [State]) {
        self.sol_level += 1;
        if self.sol_level >= 64 {
            self.sol_level = 0;
            for s in current_states {
                sol_tree.fresh_crumb(&mut s.sol);
            }
        }
    }

    fn solve(&mut self) {
        // Edge case where all items are in solution
        // Just return break solution
        if self.break_solution.break_item == self.item_count() {
            self.best_sol_weight = self.break_solution.weight;
            return;
        }

        let mut current_states = Vec::new();
        let mut next_states = Vec::new();
        let n = self.item_count();
        let mut i = 0;
        current_states.push(State {
            p: self.break_solution.profit,
            w: self.break_solution.weight,
            sol: SolCrumb::new(0),
        });

        let mut sol_tree = SolTree::new();
        while !current_states.is_empty() && i < n {
            self.print_update(i, &current_states, &next_states, &sol_tree);

            if self.t < n - 1 {
                self.t += 1;
                self.add_item_t(&current_states, &mut next_states);
                self.swap_state_buffers(&mut current_states, &mut next_states);
                self.backup_solution_history(&mut sol_tree, &mut current_states);
                i += 1;
            }

            if self.best_sol_weight == self.problem.capacity {
                break;
            }

            if self.s > 0 {
                self.s -= 1;
                self.remove_item_s(&current_states, &mut next_states);
                self.swap_state_buffers(&mut current_states, &mut next_states);
                self.backup_solution_history(&mut sol_tree, &mut current_states);
                i += 1;
            }

            if self.best_sol_weight == self.problem.capacity {
                break;
            }
        }
        self.print_final_update(i, &current_states, &next_states, &sol_tree);
        self.backtrack_decision(&sol_tree);
    }
}

pub fn solve(problem: &Problem) -> Result<Solution, Box<dyn std::error::Error>> {
    let mut instance = Instance::new(problem);
    instance.solve();
    Ok(Solution {
        decision: instance.decision,
        value: instance.lower_bound + instance.base_value,
        weight: instance.best_sol_weight,
    })
}
