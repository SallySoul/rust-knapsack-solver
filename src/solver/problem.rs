#[derive(Copy, Clone)]
pub struct Item {
    pub id: usize,
    pub value: usize,
    pub weight: usize,
}

impl Item {
    fn new(id: usize, value: usize, weight: usize) -> Item {
        Item { id, value, weight }
    }
}

pub struct Problem {
    pub items: Vec<Item>,
    pub capacity: usize,
}

impl Problem {
    pub fn read<F: std::io::BufRead>(input: F) -> Result<Problem, Box<dyn std::error::Error>> {
        let mut lines = input.lines();

        let item_count = match lines.next() {
            Some(token) => token?.parse::<usize>()?,
            None => panic!("Should be usize for item count"),
        };

        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            if let Some(Ok(tokens_line)) = lines.next() {
                let mut tokens = tokens_line.split_whitespace();
                let id = tokens.next().unwrap().parse::<usize>()?;
                let value = tokens.next().unwrap().parse::<usize>()?;
                let weight = tokens.next().unwrap().parse::<usize>()?;
                items.push(Item::new(id, value, weight));
            } else {
                panic!("Should be usize for item count");
            }
        }

        let capacity = match lines.next() {
            Some(token) => token?.parse::<usize>()?,
            None => panic!("Should be usize for capacity"),
        };

        Ok(Problem { items, capacity })
    }
}

pub struct Solution {
    pub decision: Vec<bool>,
    pub value: usize,
    pub weight: usize,
}

impl Solution {
    pub fn validate(&self, problem: &Problem) -> bool {
        let mut value_sum = 0;
        let mut weight_sum = 0;
        let mut valid = true;

        if self.decision.len() != problem.items.len() {
            println!(
                "ERROR: Solution::validate decision len same size as problem, {} vs {}",
                self.decision.len(),
                problem.items.len()
            );
            valid = false;
        }

        for (d, i) in self.decision.iter().zip(problem.items.iter()) {
            if *d {
                value_sum += i.value;
                weight_sum += i.weight;
            }
        }

        if value_sum != self.value {
            println!("ERROR: Solution::validate, value does not match!");
            valid = false;
        }
        if weight_sum != self.weight {
            println!("ERROR: Solution::validate, weight does not match!");
            valid = false;
        }

        valid
    }
}
