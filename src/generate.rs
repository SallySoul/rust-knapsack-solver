use clap::arg_enum;
use clap::Parser;
use rand::distributions::Uniform;
use rand::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

arg_enum! {
#[derive(Debug)]
pub enum Correlation {
    None,
    Weak,
    Strong
}
}

#[derive(Parser, Debug)]
pub struct Options {
    /// How the weight and values of the items should correlate.
    /// Options are None, Weak, and Strong
    #[clap(long, default_value_t = Correlation::None)]
    correlation: Correlation,

    /// How many items to generate
    #[clap(short = 'n', long, value_parser, default_value_t = 30)]
    item_count: usize,

    /// Capacity for the knapsack
    #[clap(short, long, value_parser, default_value_t = 700)]
    capacity: usize,

    /// Upper bound on weight
    #[clap(short, long, value_parser, default_value_t = 100)]
    weight_bound: usize,

    /// Upper bound on weight
    #[clap(short, long, value_parser, default_value_t = 100)]
    value_bound: usize,

    /// Where to write the problem file
    #[clap(short, long, value_parser)]
    output_path: PathBuf,
}

pub fn run(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let output_file = File::create(&options.output_path)?;
    let mut output_writer = BufWriter::new(output_file);
    let mut rng = rand::thread_rng();

    writeln!(output_writer, "{}", options.item_count)?;
    match options.correlation {
        Correlation::None => write_no_correlation(options, &mut output_writer, &mut rng)?,
        Correlation::Weak => write_weak_correlation(options, &mut output_writer, &mut rng)?,
        Correlation::Strong => write_strong_correlation(options, &mut output_writer, &mut rng)?,
    }
    writeln!(output_writer, "{}", options.capacity)?;

    Ok(())
}

fn write_no_correlation<O: std::io::Write>(
    options: &Options,
    output: &mut O,
    rng: &mut ThreadRng,
) -> Result<(), Box<dyn std::error::Error>> {
    let value_distribution = Uniform::from(0..options.value_bound);
    let weight_distribution = Uniform::from(0..options.weight_bound);
    for id in 0..options.item_count {
        let value = value_distribution.sample(rng);
        let weight = weight_distribution.sample(rng);
        writeln!(output, "{} {} {}", id, value, weight)?;
    }
    Ok(())
}

fn write_weak_correlation<O: std::io::Write>(
    options: &Options,
    output: &mut O,
    rng: &mut ThreadRng,
) -> Result<(), Box<dyn std::error::Error>> {
    let t_distribution = Uniform::from(0.0..1f32);
    let offset_distribution = Uniform::from(-1.0..1.0f32);
    let value_bound_f32 = options.value_bound as f32;
    let weight_bound_f32 = options.weight_bound as f32;
    for id in 0..options.item_count {
        let value_t = t_distribution.sample(rng);
        let offset = offset_distribution.sample(rng);

        let weight_t = (value_t + 0.5 * offset).clamp(0.0, 1.0);
        let value = (value_t * value_bound_f32) as usize;
        let weight = (weight_t * weight_bound_f32) as usize;
        writeln!(output, "{} {} {}", id, value, weight)?;
    }
    Ok(())
}

fn write_strong_correlation<O: std::io::Write>(
    options: &Options,
    output: &mut O,
    rng: &mut ThreadRng,
) -> Result<(), Box<dyn std::error::Error>> {
    let t_distribution = Uniform::from(0.0..1f32);
    let offset_distribution = Uniform::from(-1.0..1.0f32);
    let value_bound_f32 = options.value_bound as f32;
    let weight_bound_f32 = options.weight_bound as f32;
    for id in 0..options.item_count {
        let value_t = t_distribution.sample(rng);
        let offset = offset_distribution.sample(rng);

        let weight_t = (value_t + 0.05 * offset).clamp(0.0, 1.0);
        let value = (value_t * value_bound_f32) as usize;
        let weight = (weight_t * weight_bound_f32) as usize;
        writeln!(output, "{} {} {}", id, value, weight)?;
    }
    Ok(())
}
