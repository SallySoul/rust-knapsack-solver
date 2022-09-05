use crate::solver::problem::*;

// Pre-emptivley catch large
// TODO: make this more sensible / configurable?
pub const MAX_STATES: usize = 1000000;

struct Array {
    data: Vec<usize>,
    width: usize,
}

impl Array {
    fn new(width: usize, height: usize) -> Array {
        let size = width * height;
        if size > MAX_STATES {
            panic!("Problem exceeds size limit");
        }

        Array {
            data: vec![0; size],
            width,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn get(&self, x: usize, y: usize) -> usize {
        let index = self.index(x, y);
        self.data[self.index(x, y)]
    }

    fn set(&mut self, x: usize, y: usize, v: usize) {
        let index = self.index(x, y);
        self.data[index] = v
    }

    fn last(&self) -> usize {
        *self.data.last().unwrap()
    }
}

pub fn solve(problem: &Problem) -> Solution {
    let width = problem.capacity + 1;
    let height = problem.items.len() + 1;
    let mut sum_array = Array::new(width, height);

    // This outer loop is looping over items
    // BUT with 1 indexing so recusion works
    for y in 1..height {
        let item = &problem.items[y - 1];
        // This loop is iterating over weights
        for x in 0..width {
            // Not taking y would be same be same value as
            // decision for previous item at this weight
            let do_not_take = sum_array.get(x, y - 1);

            // Other wise, find value for previous items
            // without weight of item we're gonna take
            let do_take = if item.weight <= x {
                sum_array.get(x - item.weight, y - 1) + item.value
            } else {
                0
            };
            let new_value = do_take.max(do_not_take);
            sum_array.set(x, y, new_value);
        }
    }

    // Back track to make decision vector
    let mut decision = vec![false; problem.items.len()];
    let mut x = problem.capacity;
    for y in (1..height).rev() {
        if sum_array.get(x, y) != sum_array.get(x, y - 1) {
            decision[y - 1] = true;
            x -= problem.items[y - 1].weight;
        }
    }

    Solution {
        decision,
        value: sum_array.last(),
    }
}
