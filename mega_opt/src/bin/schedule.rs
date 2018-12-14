extern crate mega_opt;
extern crate rand;
#[macro_use]
extern crate structopt;
extern crate chrono;

use mega_opt::*;
use std::collections::HashSet;
use std::sync::{Arc};
use structopt::StructOpt;
use chrono::Local;

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "t", long = "temp", default_value = "0.0")]
    temp: f64,

    #[structopt(short = "p", long = "penalty", default_value = "0.1")]
    penalty: f64,

    #[structopt(short = "l", long = "load")]
    load_from: String,

    #[structopt(short = "s", long = "save-to")]
    save_to: String,

    #[structopt(short = "c", long = "cand-limit", default_value = "10")]
    cand_limit: usize,

    #[structopt(short = "bl", long = "base-limit", default_value = "3.0")]
    base_limit: f64,

    #[structopt(short = "st", long = "timestamp")]
    save_timestamp: bool,

    #[structopt(short = "cf", long = "cand-file", default_value = "../inputs/cities.cand")]
    cand_file: String
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    let pi = load_pi(nodes.len());

    let mut penalty_config: PenaltyConfig = Default::default();
    penalty_config.base_penalty = opt.penalty;

    let primes = Arc::new(get_primes(nodes.len()));
    let mut tour =Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone(), penalty_config);
    //let candidates = load_candidates(opt.cand_limit);
    let candidates = load_candidates2(opt.cand_limit, &opt.cand_file);
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().enumerate().map(|(j, &c)| {
            let d = dist(nodes[i], nodes[c]);
            (c, d)
        }).collect::<Vec<_>>()
        //cc.iter().map(|&c| (c, 1.0)).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    //let candidates_w = load_candidates2(opt.cand_limit);
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.get_path().len(), candidates_w.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour.get_path(), &primes, penalty_config));
    println!("{:?}", tour.check_nodes_edges().unwrap().0);



    let mut moves = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = vec!();
    for pen_base in 1..100 {
        tour = tour.change_penalty(PenaltyConfig{base_penalty: 0.001 * pen_base as f64, ..Default::default()});
        let mut cc = 0usize;
        let mut last = 0usize;
        loop {
            if let Some((new_tour, pr)) = do_opt(&mut tour, &candidates_w, &pi, opt.temp, opt.base_limit, &format!("{}", pen_base), &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
                tour = new_tour;
                last = cc;
                moves += 1;
                if moves % 10 == 0 {

                }
            }
            cc += 1;
            if cc - last > 1_000_000 {
                break;
            }
            if cc % 1000000 == 0 {
                println!("cc {} {} {} {}", cc, tour.get_len(), tour.get_real_len(), Local::now().format("%Y-%m-%dT%H:%M:%S"));
                tour.save(&format!("{}-{}-tmp.csv", opt.save_to, pen_base as f64*0.001 + 1.0));
            }
        }
    }

}
