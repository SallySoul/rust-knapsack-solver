use crate::solver::problem::*;
use std::collections::HashSet;
//use std::fmt::Write;
/*
struct State {
    capacity: usize,
    value: usize,
}
*/

fn node_name(item_index: usize, capacity: usize, name_buffer: &mut String) {
    use std::fmt::Write;
    name_buffer.clear();
    write!(name_buffer, "i{}_c{}", item_index, capacity).unwrap();
}

pub fn solve(problem: &Problem) -> Result<Solution, Box<dyn std::error::Error>> {
    let item_count = problem.items.len();

    // TODO look into sorting this
    let order: Vec<usize> = (1..item_count + 1).collect();

    let mut sums: Vec<usize> = Vec::with_capacity(item_count);
    let mut sum = 0;
    for item_index in &order {
        let item = &problem.items[*item_index];
        sum += item.value;
        sums.push(sum);
    }

    //let output_file = File::create("/tmp/test.dot")?;
    //let mut output_writer = BufWriter::new(output_file);

    let mut current_name = String::new();
    let mut name_buffer = String::new();
    let mut current_frontier: HashSet<usize> = HashSet::new();
    current_frontier.insert(0);
    let mut next_frontier: HashSet<usize> = HashSet::new();
    //writeln!(&mut output_writer, "digraph g {{")?;
    let mut state_count = 0;
    for item_index in order.iter().rev() {
        println!(
            "Item Index: {}, frontier_size: {}, state_count: {}",
            item_index,
            current_frontier.len(),
            state_count
        );
        //writeln!(&mut output_writer, "  rank_{} [style=invisible];", item_index)?;
        let item_value = problem.items[*item_index - 1].value;
        //println!("Item Index: {}", item_index);
        state_count += current_frontier.len();
        //write!(&mut output_writer, "  {{ rank=same; {}", item_index)?;
        for capacity in &current_frontier {
            node_name(*item_index, *capacity, &mut name_buffer);
            //write!(&mut output_writer, " -> {}", name_buffer)?;
        }
        //writeln!(&mut output_writer, " [ style=invis ] }}")?;

        for capacity in current_frontier.drain() {
            //println!("  capacity: {}", capacity);
            next_frontier.insert(capacity);
            node_name(*item_index, capacity, &mut current_name);
            node_name(item_index - 1, capacity, &mut name_buffer);
            //writeln!(&mut output_writer, "  {} -> {}", current_name, name_buffer)?;

            if capacity >= item_value {
                let take_capacity = capacity - item_value;
                next_frontier.insert(take_capacity);
                node_name(item_index - 1, take_capacity, &mut name_buffer);
                //writeln!(&mut output_writer, "  {} -> {}", current_name, name_buffer)?;
            }
        }
        std::mem::swap(&mut current_frontier, &mut next_frontier);
    }
    println!(
        "Total states: {}, vs naive: {}",
        state_count,
        item_count * problem.capacity
    );

    //writeln!(&mut output_writer, "}}")?;

    Ok(Solution {
        decision: vec![false; problem.items.len()],
        value: 0,
    })
}
