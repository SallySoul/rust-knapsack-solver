mod dynamic;
mod greedy;
mod minknap;
mod problem;
mod sol_tree;
mod twopass;

use crate::solver::problem::*;

use clap::arg_enum;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

arg_enum! {
/// The different solver implementations that are available
#[derive(Parser, Debug)]
pub enum Solver {
    TwoPass,
    Greedy,
    BranchAndBound,
    Dynamic,
    MinKnap,
}
}

#[derive(Parser, Debug)]
pub struct Options {
    /// Which solver implementation to use
    #[clap(short, long, default_value_t = Solver::Greedy)]
    solver: Solver,

    /// Do no print the decision vector
    #[clap(short, long)]
    no_print_solution: bool,

    /// Problem file to try.
    /// If not specified, problem should be fed in via STD IN
    #[clap(short, long)]
    input_file: Option<PathBuf>,
}

pub fn run(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let problem;
    if let Some(input_path) = &options.input_file {
        let input_file = File::open(input_path)?;
        let input_reader = BufReader::new(input_file);
        problem = Problem::read(input_reader)?
    } else {
        let stdin = std::io::stdin();
        let input_reader = BufReader::new(stdin);
        problem = Problem::read(input_reader)?
    };

    let solution = match options.solver {
        Solver::Greedy => greedy::solve(&problem),
        Solver::Dynamic => dynamic::solve(&problem),
        Solver::TwoPass => twopass::solve(&problem)?,
        Solver::BranchAndBound => {
            panic!("not implemented");
        }
        Solver::MinKnap => minknap::solve(&problem)?,
    };

    let greedy_sol = greedy::solve(&problem);

    if !solution.validate(&problem) {
        println!("ERROR: Solution is not valid!");
    }

    if !options.no_print_solution {
        println!("Id\tDecision\tGD");
        for i in 0..problem.items.len() {
            println!(
                "{}\t{}\t{}",
                problem.items[i].id, solution.decision[i], greedy_sol.decision[i]
            );
        }
    }

    println!(
        "Solver Used: {:?}, Solution Value: {}",
        options.solver, solution.value
    );
    Ok(())
}
