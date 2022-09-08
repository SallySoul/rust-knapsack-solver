// Storing all previous states would be infeasible
// So we store a compressed version of the decision history
// 1 bit per decision
// Each state carries a u64,
// When that buffer is filled we commit it a merkle tree of shared history

use std::collections::HashSet;

struct BacktrackState<'a> {
    item_start: usize,
    item_order: &'a [usize],
    decision_vector: &'a mut [bool],
}

impl<'a> BacktrackState<'a> {
    fn new(item_order: &'a [usize], decision_vector: &'a mut [bool]) -> BacktrackState<'a> {
        BacktrackState {
            item_start: item_order.len() - 1,
            item_order,
            decision_vector,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct SolCrumb {
    recent: u64,
    previous: usize,
}

impl SolCrumb {
    pub fn new(previous: usize) -> SolCrumb {
        SolCrumb {
            recent: 0,
            previous,
        }
    }

    pub fn add_decision(&mut self, decision: bool) {
        self.recent <<= 1;
        self.recent |= decision as u64;
    }
}

pub struct SolTree {
    crumbs: Vec<SolCrumb>,
}

impl SolTree {
    pub fn new() -> SolTree {
        // Need blank in 0th address
        // So we can iterate until previous == 0
        let crumbs = vec![SolCrumb::new(0)];
        SolTree { crumbs }
    }

    fn get(&self, index: usize) -> &SolCrumb {
        &self.crumbs[index]
    }

    fn add_crumb(&mut self, crumb: SolCrumb) -> usize {
        let result = self.crumbs.len();
        self.crumbs.push(crumb);
        result
    }

    pub fn fresh_crumb(&mut self, crumb: &mut SolCrumb) {
        let previous = self.add_crumb(*crumb);
        crumb.previous = previous;
        crumb.recent = 0;
    }

    pub fn backtrack(
        &self,
        root_crumb: SolCrumb,
        level: usize,
        item_order: &[usize],
        decision_vector: &mut [bool],
    ) {
        let mut bt_state = BacktrackState {
            item_start: item_order.len() - 1,
            item_order,
            decision_vector,
        };

        backtrack_crumb(root_crumb.recent, level, &mut bt_state);
        let mut previous_crumb = root_crumb.previous;
        while previous_crumb != 0 {
            let crumb = self.get(previous_crumb);
            backtrack_crumb(crumb.recent, 64, &mut bt_state);
            previous_crumb = crumb.previous;
        }
    }
}

fn backtrack_crumb(recent: u64, level: usize, bt_state: &mut BacktrackState) {
    let mut binary_decisions = recent;
    for i in 0..level {
        // Pull the last bit, toggle that decision if true
        let decision = (binary_decisions & 1u64) != 0;
        binary_decisions >>= 1;
        let index = bt_state.item_order[bt_state.item_start];
        bt_state.decision_vector[index] ^= decision;

        // TODO(rbentley): after testing, replace this with
        // sub with overflow
        if bt_state.item_start > 0 {
            bt_state.item_start -= 1;
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    // Quick check of sol crumb backtracking
    // only partially full
    // Flip false decisions
    #[test]
    fn sol_crumb_1() {
        let mut sc = SolCrumb::new(0);
        sc.add_decision(true);
        assert_eq!(sc.recent, 0b1);

        sc.add_decision(true);
        assert_eq!(sc.recent, 0b11);

        sc.add_decision(false);
        assert_eq!(sc.recent, 0b110);

        sc.add_decision(true);
        assert_eq!(sc.recent, 0b1101);

        let item_order: Vec<usize> = (0..4).collect();
        let mut decision_vector = vec![false; 4];
        let mut bt_state = BacktrackState::new(&item_order, &mut decision_vector);

        backtrack_crumb(sc.recent, 4, &mut bt_state);
        assert_eq!(decision_vector, vec![true, true, false, true]);
    }

    // Check of full sol_crumb
    // flip true decisions
    #[test]
    fn sol_crumb_2() {
        let mut sc = SolCrumb::new(0);
        // We're gonna flip for this test
        // start with all trues, flip in decision
        let mut correct_decision = Vec::with_capacity(64);
        for _ in 0..32 {
            sc.add_decision(false);
            correct_decision.push(true);

            sc.add_decision(true);
            correct_decision.push(false);
        }
        assert_eq!(correct_decision.len(), 64);

        let item_order: Vec<usize> = (0..64).collect();
        let mut decision_vector = vec![true; 64];
        let mut bt_state = BacktrackState::new(&item_order, &mut decision_vector);
        backtrack_crumb(sc.recent, 64, &mut bt_state);
        assert_eq!(decision_vector, correct_decision);
    }

    // Check of sol tree backtracking
    #[test]
    fn sol_tree_1() {
        let item_count = 342;
        let mut correct_decision = Vec::with_capacity(item_count);
        let mut root_crumb = SolCrumb::new(0);
        let mut level = 0;
        let mut sol_tree = SolTree::new();
        for _ in 0..(item_count / 2) {
            root_crumb.add_decision(false);
            correct_decision.push(false);

            root_crumb.add_decision(true);
            correct_decision.push(true);

            level += 2;
            if level >= 64 {
                level = 0;
                sol_tree.fresh_crumb(&mut root_crumb);
            }
        }

        assert_eq!(level, item_count % 64);
        assert_eq!(sol_tree.crumbs.len(), (item_count / 64) + 1);

        let mut decision_vector = vec![false; item_count];
        let item_order: Vec<usize> = (0..item_count).collect();
        sol_tree.backtrack(root_crumb, level, &item_order, &mut decision_vector);
        assert_eq!(decision_vector, correct_decision);
    }

    // Check of sol tree backtracking
    // With a minknap like order
    #[test]
    fn sol_tree_2() {
        let item_count = 759;
        let mut correct_decision = Vec::with_capacity(item_count);
        let mut item_order = Vec::with_capacity(item_count);
        let mut root_crumb = SolCrumb::new(0);
        let mut sol_tree = SolTree::new();

        let b = 312;
        let mut s = b;
        let mut t = b - 1;
        let mut level = 0;
        while s > 0 || t < item_count - 1 {
            if s > 0 {
                s -= 1;
                item_order.push(s);
                correct_decision.push(true);
                root_crumb.add_decision(true);
                level += 1;
                if level >= 64 {
                    level = 0;
                    sol_tree.fresh_crumb(&mut root_crumb);
                }
            }

            if t < item_count - 1 {
                t += 1;
                item_order.push(t);
                correct_decision.push(false);
                root_crumb.add_decision(false);
                level += 1;
                if level >= 64 {
                    level = 0;
                    sol_tree.fresh_crumb(&mut root_crumb);
                }
            }
        }

        assert_eq!(level, item_count % 64);
        assert_eq!(sol_tree.crumbs.len(), (item_count / 64) + 1);

        let mut decision_vector = vec![false; item_count];
        let item_order: Vec<usize> = (0..item_count).collect();
        sol_tree.backtrack(root_crumb, level, &item_order, &mut decision_vector);
        assert_eq!(decision_vector, correct_decision);
    }
}
