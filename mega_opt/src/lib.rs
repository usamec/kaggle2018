#[macro_use] extern crate assert_approx_eq;
extern crate rand;
extern crate chrono;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::stdout;
use std::collections::HashSet;
use std::iter::FromIterator;
use chrono::Local;
use std::cell::RefCell;
use rand::prelude::*;
use std::io::Write;

mod tour;

pub use tour::Tour;
pub use tour::BareTour;

#[derive(Clone, Copy)]
pub struct PenaltyConfig {
    pub base_penalty: f64,
    pub length_slope: f64,
    pub length_min_slope: f64,
    pub big_cutoff: f64
}

impl Default for PenaltyConfig {
    fn default() -> Self {
        PenaltyConfig { base_penalty: 0.1, length_slope: 0.0, length_min_slope: 0.0, big_cutoff: 50.0 }
    }
}

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

pub fn load_candidates2(cand_limit: usize, cand_file: &str) -> Vec<Vec<usize>> {
    let f = File::open(cand_file).expect("file not found");
    let file = BufReader::new(&f);
    let mut out: Vec<Vec<usize,>> = Vec::new();
    for line in file.lines().skip(1) {
        let cur_line = line.unwrap();
        let parts = cur_line.trim().split(" ").skip(3).collect::<Vec<_>>();
        out.push(parts.chunks(2).map(|x| {
            //println!("{:?}", x);
            x[0].parse::<usize>().unwrap() - 1
        }).take(cand_limit).collect());

        //println!("next");

        /*let part2 = cur_line.skip(3).next().unwrap();
        out.push(part2.split(" ").map(|x| x.parse().unwrap()).take(cand_limit).collect());*/
    }

    out
}

pub fn load_pi(n_nodes: usize) -> Vec<f64> {
    let mut out = vec![0.0; n_nodes];
    let f = File::open("../inputs/cities.pi").expect("file not found");
    let file = BufReader::new(&f);
    for line in file.lines().skip(1) {
        let cur_line = line.unwrap();
        let parts = cur_line.trim().split(" ").collect::<Vec<_>>();
        if parts.len() == 2 {
            out[parts[0].parse::<usize>().unwrap() - 1] = parts[1].parse::<f64>().unwrap() / 100_000.0;
        }
    }
    println!("top pi {:?}", &out[..10]);
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

pub fn dist_2(nodes: &[(f64, f64)], a: usize, b: usize) -> f64 {
    dist(nodes[a], nodes[b])
}

pub fn dist_pi(pi: &[f64], nodes: &[(f64, f64)], a: usize, b: usize) -> f64 {
    dist(nodes[a], nodes[b]) + pi[a] + pi[b]
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

pub fn get_penalty(current_len: f64, cur_pos: usize, cur_node: usize, primes: &[bool], penalty_config: PenaltyConfig) -> f64 {
    if cur_pos % 10 == 0 && !primes[cur_node] {
        current_len * (penalty_config.base_penalty + (current_len - penalty_config.length_min_slope).max(0.0) * penalty_config.length_slope)
    } else {
        0.0
    }
}

// Return len with current settings, and len with competion settings
pub fn verify_and_calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool], penalty_config: PenaltyConfig) -> (f64, f64) {
    assert!(tour.len() == nodes.len() + 1);
    assert!(tour[0] == 0);
    assert!(tour[tour.len() - 1] == 0);

    let tour_set: HashSet<usize> = HashSet::from_iter(tour.iter().cloned());

    assert!(tour_set.len() == nodes.len());

    let mut total_len = 0f64;
    let mut total_real_len = 0f64;

    for i in 0..tour.len()-1 {
        let current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        total_len += current_len;
        total_len += get_penalty(current_len, i + 1, tour[i], primes, penalty_config);

        total_real_len += current_len;
        if (i + 1) % 10 == 0 && !primes[tour[i]] {
            total_real_len += 0.1 * current_len;
        }
    }
    (total_len, total_real_len)
}

pub fn calculate_len(nodes: &[(f64, f64)], tour: &[usize], primes: &[bool], offset: usize, penalty_config: PenaltyConfig) -> f64 {
    let mut total_len = 0f64;

    for i in 0..tour.len()-1 {
        let mut current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        current_len += get_penalty(current_len, i + 1 + offset, tour[i], primes, penalty_config);
        total_len += current_len;
    }
    total_len
}

fn patch3(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], temp: f64, base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>, mut all_cycle_parts: Vec<Vec<(usize, usize)>>, mut added_sum: f64, mut removed_sum: f64) -> Option<(Tour, f64)> {
    all_cycle_parts.shuffle(&mut rand::thread_rng());
    let cycle_parts = &all_cycle_parts[0];
    let cycle_partsb = &all_cycle_parts[1];

    // patch with 3opt
    for cp in cycle_parts {
        if cp.1 - cp.0 < 3 {
            continue
        }
        for s in cp.0..cp.1 {
            let v1 = tour.get_path()[s];
            let v2 = tour.get_path()[s + 1];
            for &(c1, _) in &candidates[v1] {
                if c1 == v2 {
                    continue;
                }

                let i1 = tour.get_inv()[c1];
                if cycle_parts.iter().any(|cpx| i1 > cpx.0 && i1 < cpx.1) {
                    continue
                }

                if i1 <= 2 || i1 >= tour.get_path().len() - 2 {
                    continue
                }

                if cycle_partsb.iter().any(|cpx| i1 > cpx.0 && i1 < cpx.1) {
                    for &(c2, _) in &candidates[v2] {
                        if c2 == v1 {
                            continue;
                        }
                        let i2 = tour.get_inv()[c2];
                        if cycle_parts.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                            continue
                        }
                        if cycle_partsb.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                            continue
                        }
                        if i2 <= 2 || i2 >= tour.get_path().len() - 2 {
                            continue
                        }

                        for &i12 in [i1-1, i1+1].iter() {
                            for &i22 in [i2-1, i2+1].iter() {
                                let v12 = tour.get_path()[i12];
                                let v22 = tour.get_path()[i22];

                                added.push((v1, c1));
                                added_sum += dist_pi(&pi, &tour.nodes, v1, c1);
                                added.push((v2, c2));
                                added_sum += dist_pi(&pi, &tour.nodes,  v2, c2);
                                added.push((v12, v22));
                                added_sum += dist_pi(&pi, &tour.nodes,  v12, v22);
                                removed.push((c1, v12));
                                removed_sum += dist_pi(&pi, &tour.nodes,  c1, v12);
                                removed.push((c2, v22));
                                removed_sum += dist_pi(&pi, &tour.nodes,  c2, v22);
                                removed.push((v1, v2));
                                removed_sum += dist_pi(&pi, &tour.nodes,  v1, v2);

                                if added_sum - removed_sum < base_limit {
                                    let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);

                                    if cycles == 1 {
                                        let test_fast = tour.test_changes_fast(&added, &removed);
                                        //println!("tf3 {:?} {}", test_fast, tour.get_len());
                                        if let Some(len) = test_fast {
                                            let pr = rand::thread_rng().gen::<f64>();
                                            if len < tour.get_len() || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                                                let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                                                let new_tour = tour.make_new(p, );
                                                println!("{}accept nonseq3 {} real {}, added len {} a - r {} {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                                                return Some((new_tour, pr));
                                            }
                                        }
                                    } else if cycles == 2 {
                                        if let Some(r) = patch(tour, candidates, pi, temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                            //panic!("bu");
                                            return Some(r);
                                        }
                                    } else if cycles < all_cycle_parts.len() {
                                        if let Some(r) = patch3(tour, candidates, pi, temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                            //panic!("bu");
                                            return Some(r);
                                        }
                                    }
                                }

                                added.pop();
                                added.pop();
                                added.pop();
                                removed.pop();
                                removed.pop();
                                removed.pop();

                                added_sum -= dist_pi(&pi, &tour.nodes, v1, c1);
                                added_sum -= dist_pi(&pi, &tour.nodes, v2, c2);
                                added_sum -= dist_pi(&pi, &tour.nodes, v12, v22);

                                removed_sum -= dist_pi(&pi,&tour.nodes, c1, v12);
                                removed_sum -= dist_pi(&pi,&tour.nodes, c2, v22);
                                removed_sum -= dist_pi(&pi, &tour.nodes, v1, v2);
                            }
                        }
                    }
                } else {
                    for &(c2, _) in &candidates[v2] {
                        if c2 == v1 {
                            continue;
                        }
                        let i2 = tour.get_inv()[c2];
                        if cycle_parts.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                            continue
                        }
                        if !cycle_partsb.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                            continue
                        }
                        if i2 <= 2 || i2 >= tour.get_path().len() - 2 {
                            continue
                        }

                        for &i12 in [i1-1, i1+1].iter() {
                            for &i22 in [i2-1, i2+1].iter() {
                                let v12 = tour.get_path()[i12];
                                let v22 = tour.get_path()[i22];

                                added.push((v1, c1));
                                added_sum += dist_pi(&pi, &tour.nodes, v1, c1);
                                added.push((v2, c2));
                                added_sum += dist_pi(&pi, &tour.nodes, v2, c2);
                                added.push((v12, v22));
                                added_sum += dist_pi(&pi, &tour.nodes, v12, v22);
                                removed.push((c1, v12));
                                removed_sum += dist_pi(&pi, &tour.nodes, c1, v12);
                                removed.push((c2, v22));
                                removed_sum += dist_pi(&pi, &tour.nodes, c2, v22);
                                removed.push((v1, v2));
                                removed_sum += dist_pi(&pi, &tour.nodes, v1, v2);

                                if added_sum - removed_sum < base_limit {
                                    let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);

                                    if cycles == 1 {
                                        let test_fast = tour.test_changes_fast(&added, &removed);
                                        //println!("tf3b {:?} {}", test_fast, tour.get_len());
                                        if let Some(len) = test_fast {
                                            let pr = rand::thread_rng().gen::<f64>();
                                            if len < tour.get_len() + temp.min(0.0) || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                                                let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                                                let new_tour = tour.make_new(p, );
                                                println!("{}accept nonseq3 {} real {}, added len {} a - r {} {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                                                return Some((new_tour, pr));
                                            }
                                        }
                                    } else if cycles == 2 {
                                        if let Some(r) = patch(tour, candidates, pi,temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                            //panic!("bu");
                                            return Some(r);
                                        }
                                    } else if cycles < all_cycle_parts.len() {
                                        if let Some(r) = patch3(tour, candidates,  pi,temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                            //panic!("bu");
                                            return Some(r);
                                        }
                                    }
                                }

                                added.pop();
                                added.pop();
                                added.pop();
                                removed.pop();
                                removed.pop();
                                removed.pop();

                                added_sum -= dist_pi(&pi, &tour.nodes, v1, c1);
                                added_sum -= dist_pi(&pi, &tour.nodes, v2, c2);
                                added_sum -= dist_pi(&pi, &tour.nodes, v12, v22);

                                removed_sum -= dist_pi(&pi, &tour.nodes, c1, v12);
                                removed_sum -= dist_pi(&pi, &tour.nodes, c2, v22);
                                removed_sum -= dist_pi(&pi, &tour.nodes, v1, v2);
                            }
                        }
                    }
                }
            }
        }
    }



    for cp in cycle_parts {
        if cp.1 - cp.0 < 3 {
            continue
        }
        for s in cp.0..cp.1 {
            let v1 = tour.get_path()[s];
            let v2 = tour.get_path()[s + 1];

            for &(c1, _) in &candidates[v1] {
                if c1 == v2 {
                    continue;
                }

                let i1 = tour.get_inv()[c1];
                if cycle_parts.iter().any(|cpx| i1 > cpx.0 && i1 < cpx.1) {
                    continue
                }
                for &(c2, _) in &candidates[v2] {
                    if c2 == v1 {
                        continue;
                    }

                    let i2 = tour.get_inv()[c2];
                    if cycle_parts.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                        continue
                    }


                    if i2 == i1 + 1 || i2 == i1 - 1 {
                        added.push((v1, c1));
                        added_sum += dist_pi(&pi, &tour.nodes, v1, c1);
                        added.push((v2, c2));
                        added_sum += dist_pi(&pi, &tour.nodes, v2, c2);
                        removed.push((v1, v2));
                        removed_sum += dist_pi(&pi, &tour.nodes, v1, v2);
                        removed.push((c2, c1));
                        removed_sum += dist_pi(&pi, &tour.nodes, c2, c1);

                        if added_sum - removed_sum < base_limit {
                            let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);
                            if cycles == 2 {
                                if let Some(r) = patch(tour, candidates, pi,temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                    //panic!("bu");
                                    return Some(r);
                                }
                            } else if cycles < all_cycle_parts.len() {
                                if let Some(r) = patch3(tour, candidates,  pi,temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                    //panic!("bu");
                                    return Some(r);
                                }
                            }
                        }

                        added_sum -= dist_pi(&pi, &tour.nodes, v1, c1);
                        added_sum -= dist_pi(&pi, &tour.nodes, v2, c2);
                        removed_sum -= dist_pi(&pi, &tour.nodes, v1, v2);
                        removed_sum -= dist_pi(&pi, &tour.nodes, c2, c1);
                        added.pop();
                        added.pop();
                        removed.pop();
                        removed.pop();
                    }
                }
            }
        }
    }
    None
}

fn patch(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], temp: f64, base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, _cand_buf: &mut Vec<usize>, mut all_cycle_parts: Vec<Vec<(usize, usize)>>, mut added_sum: f64, mut removed_sum: f64) -> Option<(Tour, f64)> {
    if added_sum - removed_sum > base_limit {
        return None;
    }

    let mut cycle_parts = all_cycle_parts.into_iter().next().unwrap();

    cycle_parts.iter_mut().for_each(|p| {
        *p = (p.0.min(p.1), p.0.max(p.1))
    });

    /*if cycle_parts.len() > 1 {
        return None;
    }*/

    if cycle_parts.iter().all(|x| x.1 - x.0 < 3) {
        return None;
    }

    for cp in &cycle_parts {
        if cp.1 - cp.0 < 3 {
            continue
        }
        for s in cp.0..cp.1 {
            let v1 = tour.get_path()[s];
            let v2 = tour.get_path()[s + 1];

            for &(c1, _) in &candidates[v1] {
                if c1 == v2 {
                    continue;
                }

                let i1 = tour.get_inv()[c1];
                if cycle_parts.iter().any(|cpx| i1 > cpx.0 && i1 < cpx.1) {
                    continue
                }
                for &(c2, _) in &candidates[v2] {
                    if c2 == v1 {
                        continue;
                    }

                    let i2 = tour.get_inv()[c2];
                    if cycle_parts.iter().any(|cpx| i2 > cpx.0 && i2 < cpx.1) {
                        continue
                    }


                    if i2 == i1 + 1 || i2 == i1 - 1 {
                        added.push((v1, c1));
                        added_sum += dist_pi(&pi, &tour.nodes, v1, c1);
                        added.push((v2, c2));
                        added_sum += dist_pi(&pi, &tour.nodes, v2, c2);
                        removed.push((v1, v2));
                        removed_sum += dist_pi(&pi, &tour.nodes, v1, v2);
                        removed.push((c2, c1));
                        removed_sum += dist_pi(&pi, &tour.nodes, c2, c1);

                        if added_sum - removed_sum < base_limit {
                            let test_fast = tour.test_changes_fast(&added, &removed);
                            if let Some(len) = test_fast {
                                let pr = rand::thread_rng().gen::<f64>();
                                if len < tour.get_len() + temp.min(0.0) || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr){
                                    let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                                    let new_tour = tour.make_new(p, );
                                    println!("{}accept nonseq {} real {}, added len {} a - r {} {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                                    return Some((new_tour, pr));
                                }
                            }
                        }

                        added_sum -= dist_pi(&pi, &tour.nodes, v1, c1);
                        added_sum -= dist_pi(&pi, &tour.nodes, v2, c2);
                        removed_sum -= dist_pi(&pi, &tour.nodes, v1, v2);
                        removed_sum -= dist_pi(&pi, &tour.nodes, c2, c1);
                        added.pop();
                        added.pop();
                        removed.pop();
                        removed.pop();
                    }
                }
            }
        }
    }

    None
}

thread_local!(static opt_start_v: RefCell<usize> = RefCell::new(1));

pub fn do_opt(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], temp: f64, base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>,
          tabu: &HashSet<(usize, usize)>, min_k: usize) -> Option<(Tour, f64)> {
    let mut rng = rand::thread_rng();
    /*let start_path_pos = rng.gen_range(1, tour.get_path().len() - 1);*/
    let start_path_pos: usize = opt_start_v.with(|sv| {
        *sv.borrow()
    });
    let mut start_vertex = tour.get_path()[start_path_pos];
    let mut start_vertex2 = tour.get_path()[(start_path_pos) + 1];
    opt_start_v.with(|sv| {
        let mut start_pos = sv.borrow_mut();
        *start_pos += 1;
        if *start_pos == tour.get_path().len() - 1 {
            *start_pos = 1;
        }
    });

    if rng.gen_range(0, 2) == 0 {
        let tmp = start_vertex;
        start_vertex = start_vertex2;
        start_vertex2 = tmp;
    }




    added.clear();
    removed.clear();

//    let mut removed = Vec::new();

    if tabu.contains(&(start_vertex, start_vertex2)) {
        return None;
    }

    if tabu.contains(&(start_vertex2, start_vertex)) {
        return None;
    }

    removed.push((start_vertex, start_vertex2));
//    let mut added = Vec::new();

    let mut current_vertex = start_vertex;
    let mut removed_sum = dist_pi(&pi, &tour.nodes, start_vertex, start_vertex2);
    let mut added_sum = 0.0;

    //let k = *[4].choose(&mut rng).unwrap();
    let k = 20;
    if rng.gen_range(0, 5) == 0 && min_k <= 3{
        let mut next_vertex = 0;
        cand_buf.clear();
        cand_buf.extend(candidates[current_vertex].iter().filter(|&&(c, d)| d + pi[current_vertex] + pi[c] <= removed_sum - added_sum + base_limit).map(|&x| x.0));
        for _ in 0..100 {
            let maybe_next_vertex = cand_buf.choose(&mut rng);

            if maybe_next_vertex.is_none() {
                break;
            }
            next_vertex = *maybe_next_vertex.unwrap();
            if next_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)){
                break;
            }
            next_vertex = 0;
        }
        if next_vertex == 0 {
            //println!("cannot generate add {}", k);
            return None;
        }

        let next_vertex_pos = tour.get_inv()[next_vertex];
        if tour.get_path()[next_vertex_pos+1] == 0 || tour.get_path()[next_vertex_pos-1] == 0 {
            return None;
        }

        added.push((start_vertex, next_vertex));
        added.push((start_vertex2, next_vertex));
        added_sum += dist_pi(&pi, &tour.nodes, start_vertex, next_vertex);
        added_sum += dist_pi(&pi, &tour.nodes, start_vertex2, next_vertex);

        removed.push((next_vertex, tour.get_path()[next_vertex_pos+1]));
        removed_sum += dist_pi(&pi, &tour.nodes, next_vertex, tour.get_path()[next_vertex_pos+1]);

        removed.push((next_vertex, tour.get_path()[next_vertex_pos-1]));
        removed_sum += dist_pi(&pi,&tour.nodes, next_vertex, tour.get_path()[next_vertex_pos-1]);

        added.push((tour.get_path()[next_vertex_pos+1], tour.get_path()[next_vertex_pos-1]));
        added_sum += dist_pi(&pi, &tour.nodes, tour.get_path()[next_vertex_pos-1], tour.get_path()[next_vertex_pos+1]);
        if added_sum - removed_sum < base_limit {
            let test_fast = tour.test_changes_fast(&added, &removed);

            if let Some(len) = test_fast {
                let pr = rng.gen::<f64>();
                if len < tour.get_len() + temp.min(0.0) || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                    let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                    let new_tour = tour.make_new(p, );

                    println!("{}accept 2.5 {} real {}, added len {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len());
                    stdout().flush();
                    return Some((new_tour, pr));
                } else {
                    //println!("longer {} {} {}", 2.5, len - tour.get_len(), added_sum - removed_sum);
                }
            }
        }
    } else {
        for i in 0..k {
            let mut next_vertex = 0;
            cand_buf.clear();
            cand_buf.extend(candidates[current_vertex].iter().filter(|&&(c, d)| d + pi[current_vertex] + pi[c] <= removed_sum - added_sum + base_limit).map(|&x| x.0));
            for _ in 0..100 {
                let maybe_next_vertex = cand_buf.choose(&mut rng);
                /*let maybe_next_vertex = candidates[current_vertex]
                    .choose_weighted(&mut rng, |x| {
                        if x.1 > removed_sum - added_sum + base_limit {
                            0.0
                        } else {
                            //let gain = tour.largest_dist_to_neigh(x.0) - x.1;
                            //(5.0 + gain).max(1.0)
                            //(gain / 10.0).exp()
                            1.0
                        }
                    });*/

                if maybe_next_vertex.is_none() {
                    break;
                }
                next_vertex = *maybe_next_vertex.unwrap();
                if next_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                    !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)){
                    break;
                }
                next_vertex = 0;
            }
            if next_vertex == 0 {
                //println!("cannot generate add {}", k);
                return None
            }
            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
            added.push((current_vertex, next_vertex));

            if added_sum - removed_sum > base_limit {
                //println!("out too long {}", added_sum - removed_sum);
                return None;
            }

            current_vertex = 0;

            for j in 0..3 {
                if j == 0 {
                    current_vertex = tour.rand_neighbour(next_vertex);
                } else {
                    current_vertex = tour.neighbours(next_vertex)[j-1];
                }
                if current_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                    !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)) {

                    removed.push((next_vertex, current_vertex));

                    let mut good = true;

                    added.push((current_vertex, start_vertex2));
                    let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);
                    let left = k - i - 1;
                    if cycles > left + 1 && cycles < 1_000_000 {
                        good = false;
                    }
                    /*if added.len() % 6 == 0 && cycles != 1 {
                        good = false;
                    }*/

                    if added.len() + 1 >= min_k {
                        if cycles == 2 {
                            removed_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            if let Some(r) = patch(tour, candidates, pi, temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                return Some(r);
                            }
                            added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            removed_sum -= dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                        } else if cycles == 3 && rand::thread_rng().gen_range(0, 10) == 0 {
                            removed_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            if let Some(r) = patch3(tour, candidates, pi, temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                return Some(r);
                            }
                            added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            removed_sum -= dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                        } else if cycles >= 4 && cycles <= 10 && rand::thread_rng().gen_range(0, 10) == 0 {
                            removed_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            if let Some(r) = patch3(tour, candidates,  pi,temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                                return Some(r);
                            }
                            added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            removed_sum -= dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                        }
                    }

                    added.pop();

                    if good {
                        break;
                    }
                    removed.pop();
                }
                current_vertex = 0;
            }

            if current_vertex == 0 {
                //println!("cannot generate remove {}", k);
                return None
            }

            removed_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);

            added.push((current_vertex, start_vertex2));
            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);

            if added_sum - removed_sum < base_limit && added.len() >= min_k {
                let test_fast = tour.test_changes_fast(&added, &removed);

                if let Some(len) = test_fast {
                    let pr = rng.gen::<f64>();
                    if len < tour.get_len() + temp.min(0.0) || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                        let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                        let new_tour = tour.make_new(p, );

                        println!("{}accept {} {} real {}, added len {} added - removed {} {}", log_prefix, i+2, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                        stdout().flush();
                        return Some((new_tour, pr));
                    } else {
                        //println!("longer {} {} {}", i+2, len - tour.get_len(), added_sum - removed_sum);
                    }
                }
            }

            added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
            added.pop();
        }

        added.push((current_vertex, start_vertex2));
        added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
    }
    None
}

