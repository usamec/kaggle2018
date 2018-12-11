extern crate updater;
#[macro_use]
extern crate structopt;

use updater::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "stars")]
struct Config {
    #[structopt(short = "if", long = "input-file", default_value = "../outputs/workingx.csv")]
    input_file: String
}

fn main() {
    let opt = Config::from_args();

    unsafe {
        penalty_config.base_penalty = 0.1;
    }

    let nodes = load_poses();
    let tour = load_tour(&opt.input_file);
    let primes = get_primes(nodes.len());
    println!("Hello, world! {:?} {:?}", nodes.len(), tour.len());
    println!("{:?}", &primes[..20]);
    println!("total len {:?}", verify_and_calculate_len(&nodes, &tour, &primes));

    println!("total primes {} {}", primes.iter().map(|&x| x as usize).sum::<usize>(), (primes.iter().map(|&x| x as usize).sum::<usize>() as f64) / (nodes.len() as f64));

    let mut base_len = 0.0;
    let mut penalty_len = 0.0;
    let mut primes_at_10th = 0;
    let mut nonprimes_at_10th = 0;

    let mut lens_output = BufWriter::new(File::create("lens.txt").unwrap());
    let mut path_output = BufWriter::new(File::create("path.txt").unwrap());
    let mut primes_output = BufWriter::new(File::create("primes.txt").unwrap());
    let mut tenth_output = BufWriter::new(File::create("10th.txt").unwrap());

    for i in 0..nodes.len() {
        if primes[i] {
            writeln!(primes_output, "{} {}", nodes[i].0, nodes[i].1);
        }
    }

    let pen_thresholds = vec![5.0, 10.0, 20.0, 50.0, 100.0, 150.0];
    let mut pen_counts = pen_thresholds.iter().map(|_| 0).collect::<Vec<_>>();
    let mut pen_sums = pen_thresholds.iter().map(|_| 0.0).collect::<Vec<_>>();

    for i in 0..tour.len()-1 {
        writeln!(path_output, "{} {}", nodes[tour[i]].0, nodes[tour[i]].1);
        let current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        base_len += current_len;
        if (i + 1) % 10 == 0 {
            writeln!(tenth_output, "{} {}", nodes[tour[i]].0, nodes[tour[i]].1);
            if !primes[tour[i]] {
                let cur_penalty = get_penalty(current_len, i+1, tour[i], &primes);
                for (i, t) in pen_thresholds.iter().enumerate() {
                    if cur_penalty > t*0.1 {
                        pen_counts[i] += 1;
                        pen_sums[i] += cur_penalty;
                    }
                }
                penalty_len += cur_penalty;
                writeln!(lens_output, "{} {}", current_len, cur_penalty);
                nonprimes_at_10th += 1;
            } else {
                writeln!(lens_output, "{} {}", current_len, 0);
                primes_at_10th += 1;
            }
        } else {
            writeln!(lens_output, "{} {}", current_len, 0);
        }
    }
    writeln!(path_output, "{} {}", nodes[0].0, nodes[0].1);
    println!("base len {}", base_len);
    println!("penalty len {}", penalty_len);
    println!("primes at 10th {} {}", primes_at_10th, (primes_at_10th as f64) / ((primes_at_10th + nonprimes_at_10th) as f64));
    println!("avg step {}", base_len / nodes.len() as f64);
    println!("avg step at 10th {}", 10.0 * penalty_len / nonprimes_at_10th as f64);

    for ((t, c), s) in pen_thresholds.iter().zip(pen_counts.iter()).zip(pen_sums.iter()) {
        println!(" >{} count {} sum {}", t, c, s);
    }
}
