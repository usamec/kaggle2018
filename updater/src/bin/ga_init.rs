extern crate updater;
extern crate rand;
#[macro_use]
extern crate structopt;

use updater::*;
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
use std::fs;

fn do_opt(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], temp: f64, base_limit: f64, k: usize,  log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>) -> Option<(Tour, f64)> {
    let mut rng = rand::thread_rng();
    let start_path_pos = rng.gen_range(1, tour.get_path().len() - 1);
    let start_vertex = tour.get_path()[start_path_pos];
    let start_vertex2 = tour.get_path()[start_path_pos + 1];

    added.clear();
    removed.clear();

//    let mut removed = Vec::new();
    removed.push((start_vertex, start_vertex2));
//    let mut added = Vec::new();

    let mut current_vertex = start_vertex;
    let mut removed_sum = dist(tour.nodes[start_vertex], tour.nodes[start_vertex2]);
    let mut added_sum = 0.0;

    //let k = *[4].choose(&mut rng).unwrap();
    if rng.gen_range(0, 5) == 0 {
        let mut next_vertex = 0;
        for _ in 0..100 {
            let maybe_next_vertex = candidates[current_vertex]
                .choose_weighted(&mut rng, |x| {
                    if x.1 > removed_sum - added_sum + base_limit {
                        0.0
                    } else {
                        let gain = tour.largest_dist_to_neigh(x.0) - x.1;
                        (gain / 10.0).exp()
                        //1.0
                    }
                });
            if maybe_next_vertex.is_err() {
                break;
            }
            next_vertex = maybe_next_vertex.unwrap().0;
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
        added_sum += dist(tour.nodes[start_vertex], tour.nodes[next_vertex]);
        added_sum += dist(tour.nodes[start_vertex2], tour.nodes[next_vertex]);

        removed.push((next_vertex, tour.get_path()[next_vertex_pos+1]));
        removed_sum += dist(tour.nodes[next_vertex], tour.nodes[tour.get_path()[next_vertex_pos+1]]);

        removed.push((next_vertex, tour.get_path()[next_vertex_pos-1]));
        removed_sum += dist(tour.nodes[next_vertex], tour.nodes[tour.get_path()[next_vertex_pos-1]]);

        added.push((tour.get_path()[next_vertex_pos+1], tour.get_path()[next_vertex_pos-1]));
        added_sum += dist(tour.nodes[tour.get_path()[next_vertex_pos-1]], tour.nodes[tour.get_path()[next_vertex_pos+1]]);
        if added_sum - removed_sum < base_limit {
            let test_fast = tour.test_changes_fast(&added, &removed);

            if let Some(len) = test_fast {
                let pr = rng.gen::<f64>();
                if len < tour.get_len() || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                    let (res, p) = tour.test_changes(&added, &removed).unwrap();
                    let new_tour = tour.make_new(p, );

                    //println!("{}accept 2.5 {} real {}, added len {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len());
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
            cand_buf.extend(candidates[current_vertex].iter().filter(|&&(c, d)| d <= removed_sum - added_sum + base_limit).map(|&x| x.0));
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
            added_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
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
                    let (cycles, _) = tour.count_cycles(&added, &removed);
                    let left = k - i - 1;
                    if cycles > left + 1 && cycles < 1_000_000 {
                        good = false;
                    }
                    if added.len() % 6 == 0 && cycles != 1 {
                        good = false;
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

            removed_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);

            added.push((current_vertex, start_vertex2));
            added_sum += dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);

            if added_sum - removed_sum < base_limit {
                let test_fast = tour.test_changes_fast(&added, &removed);

                if let Some(len) = test_fast {
                    let pr = rng.gen::<f64>();
                    if len < tour.get_len() || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
                        let (res, p) = tour.test_changes(&added, &removed).unwrap();
                        let new_tour = tour.make_new(p, );

                        //println!("{}accept {} {} real {}, added len {} added - removed {}", log_prefix, i+2, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum);
                        stdout().flush();
                        return Some((new_tour, pr));
                    } else {
                        //println!("longer {} {} {}", i+2, len - tour.get_len(), added_sum - removed_sum);
                    }
                }
            }

            added_sum -= dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);
            added.pop();
        }

        added.push((current_vertex, start_vertex2));
        added_sum += dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);
    }
    None
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "t", long = "temp", default_value = "0.03")]
    temp: f64,

    #[structopt(short = "p", long = "penalty", default_value = "0.1")]
    penalty: f64,

    #[structopt(short = "n", long = "n-threads", default_value = "4")]
    n_threads: usize,

    #[structopt(short = "ct", long = "cands-per-thread", default_value = "125")]
    cands_per_thread: usize,

    #[structopt(short = "i", long = "iters-per-cand", default_value = "1000000")]
    iters_per_cand: usize,

    #[structopt(short = "l", long = "load")]
    load_from: String,

    #[structopt(short = "s", long = "save-to")]
    save_to: String,

    #[structopt(short = "c", long = "cand-limit", default_value = "10")]
    cand_limit: usize,

    #[structopt(short = "bl", long = "base-limit", default_value = "3.0")]
    base_limit: f64
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    unsafe {
        penalty_config.base_penalty = opt.penalty;
    }


    let primes = Arc::new(get_primes(nodes.len()));
    let base_tour = Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone());
    //let candidates = load_candidates(opt.cand_limit);
    let candidates = load_candidates2(opt.cand_limit);
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().enumerate().map(|(j, &c)| {
            let d = dist(nodes[i], nodes[c]);
            (c, d)
        }).collect::<Vec<_>>()
        //cc.iter().map(|&c| (c, 1.0)).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    //let candidates_w = load_candidates2(opt.cand_limit);
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), base_tour.get_path().len(), candidates_w.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &base_tour.get_path(), &primes));
    println!("{:?}", base_tour.check_nodes_edges().unwrap().0);

    let mut handles = vec![];
    let temp = opt.temp;

    for thread_id in 0..opt.n_threads {
        let our_base_tour = base_tour.clone();
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let base_limit = opt.base_limit;
        let cands_per_thread = opt.cands_per_thread;
        let iter_per_cand = opt.iters_per_cand;
        let handle = thread::spawn(move || {
            for cand in 0..cands_per_thread {
                let mut our_tour = our_base_tour.clone();
                let mut cc = 0;
                let mut last_best = cc;
                let mut cur_best = our_tour.get_len();
                let mut added_v = vec!();
                let mut removed_v = vec!();
                let mut cand_buf = vec!();
                loop {
                    if let Some((new_tour, pr)) = do_opt(&mut our_tour, &our_candidates, temp, base_limit, 4, "", &mut added_v, &mut removed_v, &mut cand_buf) {
                        {
                            if new_tour.get_len() < our_tour.get_len() || (temp > 0.0 && ((our_tour.get_len() - new_tour.get_len()) / temp).exp() > pr) {
                                if new_tour.get_len() < cur_best {
                                    cur_best = new_tour.get_len();
                                    last_best = cc;
                                }
                                our_tour = new_tour;
                            }
                        }
                    }
                    cc += 1;
                    if cc % 100000 == 0 {
                        println!("cc {} {} {} / {}", cc, thread_id, our_tour.get_len(), our_tour.get_real_len());
                    }
                    if cc > last_best + iter_per_cand {
                        break;
                    }
                }
                println!("saving tour {} with len {} / {}",  thread_id*cands_per_thread+cand, our_tour.get_len(), our_tour.get_real_len());
                our_tour.save(&format!("{}/ga-{}.csv", prefix, thread_id*cands_per_thread+cand));
            }

        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
