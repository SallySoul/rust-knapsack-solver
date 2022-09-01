use clap::Parser;

mod generate;
mod solver;

#[derive(Parser, Debug)]
#[clap(version)]
enum Command {
    Generate(generate::Options),
    Solve(solver::Options),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse();

    match command {
        Command::Generate(options) => generate::run(&options)?,
        Command::Solve(options) => solver::run(&options)?,
    }

    Ok(())
}
