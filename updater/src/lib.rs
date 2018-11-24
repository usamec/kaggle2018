extern crate rand;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashSet;
use std::iter::FromIterator;

mod local_brute;
mod union_find;

pub use local_brute::full_optim;

pub use union_find::UnionFind;

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

pub fn verify_and_calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool]) -> f64 {
    assert!(tour.len() == nodes.len() + 1);
    assert!(tour[0] == 0);
    assert!(tour[tour.len() - 1] == 0);

    let tour_set: HashSet<usize> = HashSet::from_iter(tour.iter().cloned());

    assert!(tour_set.len() == nodes.len());

    let mut total_len = 0f64;

    for i in 0..tour.len()-1 {
        let mut current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        if (i + 1) % 10 == 0 && !primes[tour[i]] {
            current_len *= 1.1;
        }
        total_len += current_len;
    }
    total_len
}

pub fn calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool], offset: usize) -> f64 {
    let mut total_len = 0f64;

    for i in 0..tour.len()-1 {
        let mut current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        if (i + 1 + offset) % 10 == 0 && !primes[tour[i]] {
            current_len *= 1.1;
        }
        total_len += current_len;
    }
    total_len
}