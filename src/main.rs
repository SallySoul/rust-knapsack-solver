use clap::Parser;

mod generate;

#[derive(Parser, Debug)]
#[clap(version)]
enum Command {
    Generate(generate::Options),
    Solve,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse();

    match command {
        Command::Generate(options) => generate::run(&options)?,
        Command::Solve => println!("TO BE IMPLEMENTED"),
    }

    Ok(())
}
