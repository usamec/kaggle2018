#[macro_use] extern crate assert_approx_eq;
extern crate rand;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashSet;
use std::iter::FromIterator;

mod local_brute;
mod union_find;
mod tour;

pub use local_brute::full_optim;

pub use union_find::UnionFind;

pub use tour::Tour;

pub struct PenaltyConfig {
    pub base_penalty: f64,
    // (edge_length, edge_id) -> (0.0,1.0)
    // if not defined assumes |_,_| 1.0
    pub penalty_lambda: Option<Box<Fn(f64, usize) -> f64>>
}

pub static mut penalty_config: PenaltyConfig = PenaltyConfig { base_penalty: 0.1, penalty_lambda: None };

pub fn load_poses() -> Vec<(f64,f64)> {
    let f = File::open("../inputs/cities.csv").expect("file not found");
    let file = BufReader::new(&f);
    let mut out = Vec::new();
    for line in file.lines().skip(1) {
        let cur_line = line.unwrap();
        let parts = cur_line.split(",").collect::<Vec<_>>();
        out.push((parts[1].parse::<f64>().unwrap(), parts[2].parse::<f64>().unwrap()));
    }
    out
}

pub fn load_candidates(cand_limit: usize) -> Vec<Vec<usize>> {
    let f = File::open("../inputs/pi-nearest.txt").expect("file not found");
    let file = BufReader::new(&f);
    let mut out: Vec<Vec<usize>> = Vec::new();
    for line in file.lines() {
        let cur_line = line.unwrap();
        let part2 = cur_line.split(": ").skip(1).next().unwrap();
        out.push(part2.split(" ").map(|x| x.parse().unwrap()).take(cand_limit).collect());
    }

    /*let c2 = out.clone();
    for (i, l) in c2.iter().enumerate() {
        for &j in l {
            if !out[j].contains(&i) {
                out[j].push(i);
            }
        }
    }*/

    out
}

pub fn load_tour(path: &str) -> Vec<usize> {
    let f = File::open(path).expect("file not found");
    let file = BufReader::new(&f);
    file.lines().skip(1).map(|x| x.unwrap().parse().unwrap()).collect()
}

pub fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)).sqrt()
}

pub fn get_primes(limit: usize) -> Vec<bool> {
    let mut res = vec![true; limit];

    res[0] = false;
    res[1] = false;
    for i in 2..limit {
        if res[i] {
            for j in (2*i..limit).step_by(i) {
                res[j] = false;
            }
        }
    }
    res
}

pub fn get_penalty(current_len: f64, cur_pos: usize, cur_node: usize, primes: &[bool]) -> f64 {
    if cur_pos % 10 == 0 && !primes[cur_node] {
        current_len * unsafe { penalty_config.base_penalty } * unsafe { penalty_config.penalty_lambda.as_ref().map(|x| x(current_len, cur_pos)).unwrap_or(1.0)}
    } else {
        0.0
    }
}

pub fn verify_and_calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool]) -> f64 {
    assert!(tour.len() == nodes.len() + 1);
    assert!(tour[0] == 0);
    assert!(tour[tour.len() - 1] == 0);

    let tour_set: HashSet<usize> = HashSet::from_iter(tour.iter().cloned());

    assert!(tour_set.len() == nodes.len());

    let mut total_len = 0f64;

    for i in 0..tour.len()-1 {
        let mut current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        current_len += get_penalty(current_len, i + 1, tour[i], primes);
        total_len += current_len;
    }
    total_len
}

pub fn calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool], offset: usize) -> f64 {
    let mut total_len = 0f64;

    for i in 0..tour.len()-1 {
        let mut current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        current_len += get_penalty(current_len, i + 1 + offset, tour[i], primes);
        total_len += current_len;
    }
    total_len
}
