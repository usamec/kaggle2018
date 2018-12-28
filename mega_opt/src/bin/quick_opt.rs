extern crate mega_opt;
extern crate rand;
#[macro_use]
extern crate structopt;
extern crate chrono;

use mega_opt::*;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use rand::prelude::*;
use std::collections::HashSet;
use std::sync::{Mutex, Arc};
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use structopt::StructOpt;
use std::io::Write;
use std::io::stdout;
use std::iter;
use std::time;
use std::borrow::BorrowMut;
use std::fs;
use chrono::Local;
use std::process::Command;
use std::cell::RefCell;

/// The logistic aka sigmoid function.
#[inline]
pub fn sigmoid(f: f64) -> f64 {
    use std::f64::consts::E;
    1.0 / (1.0 + E.powf(-f))
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "t", long = "temp", default_value = "0.03")]
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

    let mut penalty_config: PenaltyConfig = Default::default();
    penalty_config.base_penalty = opt.penalty;

    let pi = load_pi(nodes.len());
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


    let mut cc = 0;
    let mut moves = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = vec!();
    /*loop {
        if let Some((new_tour, pr)) = do_opt(&mut tour, &candidates_w, &pi, opt.temp, opt.base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
            tour = new_tour;

            moves += 1;
            if moves % 10 == 0 {
                tour.save(&format!("{}-tmp.csv", opt.save_to));
            }
        }
        cc += 1;
        if cc % 100000 == 0 {
            println!("cc {} {} {}", cc, 0, Local::now().format("%Y-%m-%dT%H:%M:%S"));
        }
    }*/

    for i in 1..tour.get_path().len() - 1 {
        let res = do_opt_all_limit(&mut tour, &candidates_w, &pi, opt.base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf, i, 10);
        if res.is_some() || i % 100 == 0 {
            println!("{} {}", i, res.is_some());
        }
        if let Some(new_tour) = res {
            if new_tour.get_len() < tour.get_len() {
                tour = new_tour;
                tour.save(&format!("{}-best.csv", opt.save_to));
            }
        }

    }

    /*loop {
        if let Some((new_tour, pr)) = do_opt_ds(&mut tour, &candidates_w, &pi, opt.base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
            panic!("booo2");
            tour = new_tour;

            moves += 1;
            if moves % 10 == 0 {
                tour.save(&format!("{}-tmp.csv", opt.save_to));
            }
        }
        cc += 1;
        if cc % 100000 == 0 {
            println!("cc {} {} {}", cc, 0, Local::now().format("%Y-%m-%dT%H:%M:%S"));
        }
    }*/
}

fn fix_it(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, mut added_sum: f64, mut removed_sum: f64) -> Option<(Tour, f64)> {
    let mut removed_inds = removed.iter().map(|x| iter::once(tour.get_inv()[x.0]).chain(iter::once(tour.get_inv()[x.1]))).flatten().collect::<Vec<_>>();
    let min_removed = *removed_inds.iter().min().unwrap();
    let max_removed = *removed_inds.iter().max().unwrap();

    if max_removed - min_removed > 5000 {
        println!("not fix {} {} {}", min_removed, max_removed, added.len());
        return None;
    }

    println!("fix {} {} {}", min_removed, max_removed, added.len());


    for i in min_removed..max_removed {
        let start_vertex = tour.get_path()[i];
        let start_vertex2 = tour.get_path()[i+1];

        removed.push((start_vertex, start_vertex2));
        removed_sum += dist_pi(&pi, &tour.nodes, start_vertex, start_vertex2);

        if let Some(new_tour) = do_opt_all_inner(tour, candidates, pi, base_limit, log_prefix, added, removed, start_vertex, start_vertex2, 3, added_sum, removed_sum) {
            println!("found fix {} {}", new_tour.get_len(), tour.get_len());
            panic!("booo");
            return Some((new_tour, 0.0));
        }
        removed_sum -= dist_pi(&pi, &tour.nodes, start_vertex, start_vertex2);
        removed.pop();
    }
    None
}

thread_local!(static opt_start_v2: RefCell<usize> = RefCell::new(1));

fn do_opt_ds(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>,
              tabu: &HashSet<(usize, usize)>, min_k: usize) -> Option<(Tour, f64)> {
    let mut rng = rand::thread_rng();
    /*let start_path_pos = rng.gen_range(1, tour.get_path().len() - 1);*/
    let start_path_pos: usize = opt_start_v2.with(|sv| {
        *sv.borrow()
    });
    let mut start_vertex = tour.get_path()[start_path_pos];
    let mut start_vertex2 = tour.get_path()[(start_path_pos) + 1];
    opt_start_v2.with(|sv| {
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
    let k = 3;
    {
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

                    {
                        if cycles == 2 && added.len() == 2 {
                            removed_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
                            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
                            if let Some(r) = patch_ds(tour, candidates, pi, base_limit, log_prefix, added, removed,  cycle_parts, added_sum, removed_sum) {
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
                    if len < tour.get_len() {
                        let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                        let new_tour = tour.make_new(p, );

                        println!("{}accept {} {} real {}, added len {} added - removed {} {}", log_prefix, i+2, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                        stdout().flush();
                        return Some((new_tour, pr));
                    } else {
                        println!("maybe fix {} {}", len - tour.get_len(), added_sum - removed_sum);
                        if let Some(res) = fix_it(tour, candidates, pi, base_limit, log_prefix, added, removed, added_sum, removed_sum) {
                            return Some(res)
                        }
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

fn patch_ds(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, mut all_cycle_parts: Vec<Vec<(usize, usize)>>, mut added_sum: f64, mut removed_sum: f64) -> Option<(Tour, f64)> {
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
                                if len < tour.get_len() {
                                    let (_res, p) = tour.test_changes(&added, &removed).unwrap();
                                    let new_tour = tour.make_new(p, );
                                    println!("{}accept nonseq {} real {}, added len {} a - r {} {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum, Local::now().format("%Y-%m-%dT%H:%M:%S"));
                                    return Some((new_tour, pr));
                                } else {
                                    println!("maybe fixns {} {}", len - tour.get_len(), added_sum - removed_sum);
                                    if let Some(res) = fix_it(tour, candidates, pi, base_limit, log_prefix, added, removed, added_sum, removed_sum) {
                                        return Some(res);
                                    }
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
