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
    Some,
}
}

#[derive(Parser, Debug)]
pub struct Options {
    /// How the weight and values of the items should correlate.
    /// Options are None, and Some
    /// If choosing Some, use the coeff argument to determine
    /// ammount of correlation
    #[clap(long, default_value_t = Correlation::None)]
    correlation: Correlation,

    #[clap(long, value_parser, default_value_t = 0.1)]
    coeff: f32,

    /// How many items to generate
    #[clap(short = 'n', long, value_parser, default_value_t = 30)]
    item_count: usize,

    /// Capacity for the knapsack,
    /// If unspecified, use the weight sum proportion
    #[clap(short, long, value_parser)]
    capacity: Option<usize>,

    /// Capacity, by default is proportion of sum of item weights
    #[clap(long, value_parser, default_value_t = 0.5)]
    capacity_ratio: f32,

    /// Upper bound on weight
    #[clap(short, long, value_parser, default_value_t = 100)]
    weight_bound: usize,

    /// Upper bound on weight
    #[clap(short, long, value_parser, default_value_t = 100)]
    value_bound: usize,

    /// Lower bound for values. between 0 and 1.0
    /// scaled with value_bound
    #[clap(long, value_parser, default_value_t = 0.0)]
    value_t_lower_bound: f32,

    /// Where to write the problem file
    #[clap(short, long, value_parser)]
    output_path: PathBuf,
}

pub fn run(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let output_file = File::create(&options.output_path)?;
    let mut output_writer = BufWriter::new(output_file);
    let mut rng = rand::thread_rng();

    writeln!(output_writer, "{}", options.item_count)?;
    let weight_sum = match options.correlation {
        Correlation::None => write_no_correlation(options, &mut output_writer, &mut rng)?,
        Correlation::Some => write_correlation(options, &mut output_writer, &mut rng)?,
    };

    let capacity = if let Some(c) = options.capacity {
        c
    } else {
        (options.capacity_ratio * weight_sum as f32).ceil() as usize
    };
    println!("Weight Sum: {}, Capacity: {}", weight_sum, capacity);
    writeln!(output_writer, "{}", capacity)?;

    Ok(())
}

fn write_no_correlation<O: std::io::Write>(
    options: &Options,
    output: &mut O,
    rng: &mut ThreadRng,
) -> Result<usize, Box<dyn std::error::Error>> {
    let value_distribution = Uniform::from(0..options.value_bound);
    let weight_distribution = Uniform::from(0..options.weight_bound);
    let mut weight_sum = 0;
    for id in 0..options.item_count {
        let value = value_distribution.sample(rng);
        let weight = weight_distribution.sample(rng);
        weight_sum += weight;
        writeln!(output, "{} {} {}", id, value, weight)?;
    }
    Ok(weight_sum)
}

fn write_correlation<O: std::io::Write>(
    options: &Options,
    output: &mut O,
    rng: &mut ThreadRng,
) -> Result<usize, Box<dyn std::error::Error>> {
    let t_distribution = Uniform::from(0.0..1f32);
    let offset_distribution = Uniform::from(-1.0..1.0f32);
    let value_bound_f32 = options.value_bound as f32;
    let weight_bound_f32 = options.weight_bound as f32;
    let mut weight_sum = 0;
    for id in 0..options.item_count {
        let value_t = t_distribution.sample(rng);
        let offset = offset_distribution.sample(rng);

        let weight_t = (value_t + value_t * options.coeff * offset).clamp(0.0, 1.0);
        let value = (value_t * value_bound_f32) as usize;
        // No zero weights
        let weight = 1.max((weight_t * weight_bound_f32) as usize);

        // no zero weights!
        weight_sum += weight;

        if weight == 0 {
            println!(
                "vt: {}, wt: {}, v: {}, w: {}",
                value_t, weight_t, value, weight
            );
        }
        writeln!(output, "{} {} {}", id, value, weight)?;
    }
    Ok(weight_sum)
}
