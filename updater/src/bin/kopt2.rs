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

/// The logistic aka sigmoid function.
#[inline]
pub fn sigmoid(f: f64) -> f64 {
    use std::f64::consts::E;
    1.0 / (1.0 + E.powf(-f))
}

fn do_opt2(tour: &mut Tour, candidates: &[Vec<(usize, f64)>]) -> Option<Tour> {
    let mut rng = rand::thread_rng();
    let start_path_pos = rng.gen_range(1, tour.get_path().len() - 1);
    let start_vertex = tour.get_path()[start_path_pos];
    let start_vertex2 = tour.get_path()[start_path_pos+1];

    let mut removed = Vec::new();
    removed.push((start_vertex, start_vertex2));
    let mut added = Vec::new();

    let mut current_vertex = start_vertex;
    let mut removed_sum = dist(tour.nodes[start_vertex], tour.nodes[start_vertex2]);
    let mut added_sum = 0.0;
    for i in 0..*[1,2].choose(&mut rng).unwrap() {
        let mut next_vertex = 0;
        for _ in 0..100 {
            next_vertex = candidates[current_vertex].choose_weighted(&mut rng, |x| x.1).unwrap().0;
            if next_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)) {
                if i != 0 || (tour.get_inv()[current_vertex] as i32 - tour.get_inv()[next_vertex] as i32).abs() > 100 {
                    break;
                }
            }
            next_vertex = 0;
        }
        if next_vertex == 0 {
            return None;
        }
        added_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
        added.push((current_vertex, next_vertex));

        /*if added_sum - removed_sum > 5.0 {
            return None
        }*/


        current_vertex = 0;
        for _ in 0..100 {
            current_vertex = tour.rand_neighbour(next_vertex);
            if current_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)) {
                break;
            }
            current_vertex = 0;
        }
        if current_vertex == 0 {
            return None;
        }

        removed_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
        removed.push((next_vertex, current_vertex));
    }

    added.push((current_vertex, start_vertex2));
    added_sum += dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);

    if added_sum - removed_sum > 1.0 {
        return None
    }



    let test_fast = tour.test_changes_fast(&added, &removed);
    if test_fast.is_none() {
        return None;
    }

    let mut actual_len = test_fast.unwrap();
    let start_len = tour.get_len();

    if actual_len > start_len + 50.0 {
        return None;
    }

    let (_, cur_tour_path) = tour.test_changes(&added, &removed).unwrap();
    let mut cur_tour = tour.make_new(cur_tour_path);
    let mut actual_real_len = cur_tour.get_real_len();

    let removed_inds = removed.iter().map(|x| iter::once(tour.get_inv()[x.0]).chain(iter::once(tour.get_inv()[x.1]))).flatten().collect::<Vec<_>>();

    let min_removed = *removed_inds.iter().min().unwrap();
    let max_removed = *removed_inds.iter().max().unwrap();

    println!("go {} {} {} {} {} {}", added.len(), added_sum - removed_sum, start_len, actual_len, min_removed, max_removed);


    let mut last = 0;
    let mut fouls = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = vec!();
    for i in 0..1_000_000_000 {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, 0.0, 3.0, "heavy-", &mut added_v, &mut removed_v, &mut cand_buf) {
            if new_tour.get_path() != tour.get_path() {
                if new_tour.get_len() < actual_len {
                    actual_len = new_tour.get_len();
                    actual_real_len = new_tour.get_real_len();
                    println!("bet {} {}", actual_len, start_len);
                    last = i;
                    cur_tour = new_tour;
                    fouls = 0;
                }
            } else {
                fouls += 1;
                if fouls == 10 {
                    break;
                }
            }
        }
        if i - last > 1_000_000 {
            break;
        }
    }
    if actual_len < start_len {
        println!("acceptx {} real {} added - removed {}", actual_len, actual_real_len, added_sum - removed_sum);
        stdout().flush();
        Some(cur_tour)
    } else {
        None
    }
}

fn patch(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], temp: f64, base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>, mut cycle_parts: Vec<(usize, usize)>, mut added_sum: f64, mut removed_sum: f64) -> Option<(Tour, f64)> {
    if added_sum - removed_sum > base_limit {
        return None;
    }
    cycle_parts.iter_mut().for_each(|p| {
        *p = (p.0.min(p.1), p.0.max(p.1))
    });

    if cycle_parts[0].1 - cycle_parts[0].0 < 3 || cycle_parts.len() > 1 {
        return None;
    }

    for s in cycle_parts[0].0..cycle_parts[0].1 {
        let v1 = tour.get_path()[s];
        let v2 = tour.get_path()[s+1];

        for &(c1, _) in &candidates[v1] {
            if c1 == v2 {
                continue;
            }

            let i1 = tour.get_inv()[c1];
            if i1 > cycle_parts[0].0 && i1 < cycle_parts[0].1 {
                continue
            }
            for &(c2, _) in &candidates[v2] {
                if c2 == v1 {
                    continue;
                }

                let i2 = tour.get_inv()[c2];
                if i2 > cycle_parts[0].0 && i2 < cycle_parts[0].1 {
                    continue
                }


                if i2 == i1 + 1 || i2 == i1 - 1 {
                    added.push((v1, c1));
                    added_sum += dist(tour.nodes[v1], tour.nodes[c1]);
                    added.push((v2, c2));
                    added_sum += dist(tour.nodes[v2], tour.nodes[c2]);
                    removed.push((v1, v2));
                    removed_sum += dist(tour.nodes[v1], tour.nodes[v2]);
                    removed.push((c2, c1));
                    removed_sum += dist(tour.nodes[c2], tour.nodes[c1]);

                    let test_fast = tour.test_changes_fast(&added, &removed);
                    if let Some(len) = test_fast {
                        if len < tour.get_len() {
                            let (res, p) = tour.test_changes(&added, &removed).unwrap();
                            let new_tour = tour.make_new(p, );
                            println!("{}accept nonseq {} real {}, added len {} a - r {}", log_prefix, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum);
                            return Some((new_tour, rand::thread_rng().gen::<f64>()));
                        }
                    }

                    added_sum -= dist(tour.nodes[v1], tour.nodes[c1]);
                    added_sum -= dist(tour.nodes[v2], tour.nodes[c2]);
                    removed_sum -= dist(tour.nodes[v1], tour.nodes[v2]);
                    removed_sum -= dist(tour.nodes[c2], tour.nodes[c1]);
                    added.pop();
                    added.pop();
                    removed.pop();
                    removed.pop();
                }
            }
        }
    }

    None
}

fn do_opt(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], temp: f64, base_limit: f64, log_prefix: &str, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, cand_buf: &mut Vec<usize>) -> Option<(Tour, f64)> {
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
    let k = 20;
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
                    let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);
                    let left = k - i - 1;
                    if cycles > left + 1 && cycles < 1_000_000 {
                        good = false;
                    }
                    if added.len() % 6 == 0 && cycles != 1 {
                        good = false;
                    }

                    /*if cycles == 2 {
                        removed_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
                        added_sum += dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);
                        if let Some(r) = patch(tour, candidates, temp, base_limit, log_prefix, added, removed, cand_buf, cycle_parts, added_sum, removed_sum) {
                            return Some(r);
                        }
                        added_sum -= dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);
                        removed_sum -= dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
                    }*/

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

                        println!("{}accept {} {} real {}, added len {} added - removed {}", log_prefix, i+2, new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum);
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

fn local_update<F>(size: usize, start: usize, temp: f64, nodes: &[(f64, f64)], current_tour: &[usize], cur_len: f64, primes: &[bool],
                   inner: F) -> Option<(Vec<usize>, f64)>
    where F: Fn(&mut [usize], &[(f64, f64)], &[bool], usize) -> bool {
    let end = start + size;

    let mut slice_and_padding = current_tour[start - 1..end + 1].to_owned();
    let old_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
    let mut rng = rand::thread_rng();
    if inner(&mut slice_and_padding, &nodes, &primes, start - 1) {
        let new_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
        if new_len < old_len || (temp > 0.0 && ((old_len - new_len) / temp).exp() > rng.gen::<f64>()) {
            println!("boom {} {} {} {}", old_len, new_len, old_len - new_len, size);
            let mut new_tour = current_tour.to_vec();
            {
                let slice = &mut new_tour[start - 1..end + 1];
                for i in 0..slice_and_padding.len() {
                    slice[i] = slice_and_padding[i];
                }
            }
            let (new_len, new_real_len) = verify_and_calculate_len(&nodes, &new_tour, &primes);
            println!("new_len {} real {}", new_len, new_real_len);
            stdout().flush();
            if new_len < cur_len {
                println!("better {:?}", new_len);
            }
            Some((new_tour, new_len))
        } else {
            println!("wat {} {}", new_len, old_len);
            None
        }
    } else {
        None
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "t", long = "temp", default_value = "0.03")]
    temp: f64,

    #[structopt(short = "p", long = "penalty", default_value = "0.1")]
    penalty: f64,

    #[structopt(short = "n", long = "n-threads", default_value = "2")]
    n_threads: usize,

    #[structopt(short = "nb", long = "n-brute-threads", default_value = "1")]
    n_brute_threads: usize,

    #[structopt(short = "nh", long = "n-heavy-threads", default_value = "1")]
    n_heavy_threads: usize,

    #[structopt(short = "l", long = "load")]
    load_from: String,

    #[structopt(short = "s", long = "save-to")]
    save_to: String,

    #[structopt(short = "c", long = "cand-limit", default_value = "10")]
    cand_limit: usize,

    #[structopt(short = "pt", long = "penalty-threshold", default_value = "0.0")]
    penalty_threshold: f64,

    #[structopt(short = "mp", long = "min-penalty", default_value = "0.0")]
    min_penalty: f64,

    #[structopt(short = "bl", long = "base-limit", default_value = "3.0")]
    base_limit: f64
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    unsafe {
        penalty_config.base_penalty = opt.penalty;

        if opt.penalty_threshold > 0.0 {
            let threshold = opt.penalty_threshold;
            let min_penalty = opt.min_penalty;

            penalty_config.penalty_lambda = Some(
                Box::new(move |len, pos| {
                    sigmoid(((len / (threshold + 1e-10)) - 1.0) * 5.0)
                })
            );
        }
    }


    let primes = Arc::new(get_primes(nodes.len()));
    let tour = Arc::new(Mutex::new(Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone())));
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
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.lock().unwrap().get_path().len(), candidates_w.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour.lock().unwrap().get_path(), &primes));
    println!("{:?}", tour.lock().unwrap().check_nodes_edges().unwrap().0);

    let tour_hash = Arc::new(AtomicUsize::new(tour.lock().unwrap().hash()));

    let mut handles = vec![];
    let temp = opt.temp;
    println!("temp {}", temp);
    for thread_id in 0..opt.n_heavy_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some(new_tour) = do_opt2(&mut our_tour, &our_candidates) {
                    //println!("new len {}", new_tour.get_len());
                    {
                        let mut main_tour = main_tour_mutex.lock().unwrap();
                        if new_tour.get_len() < main_tour.get_len() {
                            our_tour = new_tour;
                            our_tour_hash = our_tour.hash();


                            *main_tour = our_tour.clone();
                            main_tour_hash.store(our_tour_hash, Ordering::Relaxed);
                        }
                    }
                    //our_tour.save(&format!("{}-{}.csv", prefix, thread_id));
                }
                cc += 1;
                if cc % 1000000 == 0 {
                    println!("cc {} {}", cc, thread_id);
                }
                if main_tour_hash.load(Ordering::Relaxed) != our_tour_hash {
                    println!("reload {} {}", thread_id, cc);
                    let main_tour = main_tour_mutex.lock().unwrap();
                    our_tour = main_tour.clone();
                    our_tour_hash = our_tour.hash();
                }

            }
        });
        handles.push(handle);
    }

    for thread_id in opt.n_heavy_threads..opt.n_threads + opt.n_heavy_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let base_limit = opt.base_limit;
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            let mut added_v = vec!();
            let mut removed_v = vec!();
            let mut cand_buf = vec!();
            loop {
                if let Some((new_tour, pr)) = do_opt(&mut our_tour, &our_candidates, temp, base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf) {
                    {
                        let mut main_tour = main_tour_mutex.lock().unwrap();
                        if new_tour.get_len() < main_tour.get_len() || (temp > 0.0 && ((main_tour.get_len() - new_tour.get_len()) / temp).exp() > pr) {
                            our_tour = new_tour;
                            our_tour_hash = our_tour.hash();


                            *main_tour = our_tour.clone();
                            main_tour_hash.store(our_tour_hash, Ordering::Relaxed);
                        }
                    }
                    //our_tour.save(&format!("{}-{}.csv", prefix, thread_id));
                }
                cc += 1;
                if cc % 10000 == 0 {
                    println!("cc {} {}", cc, thread_id);
                }
                if main_tour_hash.load(Ordering::Relaxed) != our_tour_hash {
                    println!("reload {} {}", thread_id, cc);
                    let main_tour = main_tour_mutex.lock().unwrap();
                    our_tour = main_tour.clone();
                    our_tour_hash = our_tour.hash();
                }

            }
        });
        handles.push(handle);
    }



    for thread_id in opt.n_threads + opt.n_heavy_threads..opt.n_threads + opt.n_heavy_threads + opt.n_brute_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let our_nodes = nodes.clone();
        let our_primes = primes.clone();
        let main_tour_hash = Arc::clone(&tour_hash);
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().get_path().to_vec();
            let (mut cur_len, mut cur_real_len) = verify_and_calculate_len(&our_nodes, &our_tour, &our_primes);
            let mut rng = rand::thread_rng();
            loop {
                let size = rng.gen_range(20, 41);
                let start = rng.gen_range(1, our_tour.len() - size - 1);
                let maybe_new_tour = local_update(size, start, 0.0, &our_nodes, &our_tour, cur_len, &our_primes, full_optim);
                if let Some((new_tour, new_len)) = maybe_new_tour {
                    println!("better brute {} {}", cur_len, new_len);
                    {
                        let mut main_tour = main_tour_mutex.lock().unwrap();
                        if new_len < main_tour.get_len() {
                            our_tour = new_tour;
                            cur_len = new_len;

                            *main_tour = Tour::new(our_tour.clone(), our_nodes.clone(), our_primes.clone());
                            main_tour_hash.store(main_tour.hash(), Ordering::Relaxed);
                        }
                    }
                }
                cc += 1;
                let main_tour = main_tour_mutex.lock().unwrap();
                our_tour = main_tour.get_path().to_vec();
                let (ccur_len, ccur_real_len) = verify_and_calculate_len(&our_nodes, &our_tour, &our_primes);
                cur_len = ccur_len;
                cur_real_len = ccur_real_len;
                println!("ccb {}", cc);
            }
        });
        handles.push(handle);
    }

    // writer thread
    {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let prefix = opt.save_to.clone();
        let handle = thread::spawn(move || {
            let mut our_tour_hash = main_tour_hash.load(Ordering::Relaxed);
            let mut best_len = main_tour_mutex.lock().unwrap().get_len();
            let mut best_real_len = main_tour_mutex.lock().unwrap().get_real_len();

            loop {
                let cur_hash = main_tour_hash.load(Ordering::Relaxed);
                if cur_hash != our_tour_hash {
                    println!("saving");
                    let main_tour = main_tour_mutex.lock().unwrap().clone();
                    main_tour.save(&format!("{}-tmp.csv", prefix));
                    fs::rename(&format!("{}-tmp.csv", prefix), &format!("{}-latest.csv", prefix));
                    if main_tour.get_len() < best_len {
                        fs::copy(&format!("{}-latest.csv", prefix), &format!("{}-best.csv", prefix));
                        best_len = main_tour.get_len();
                    }

                    if main_tour.get_real_len() < best_real_len {
                        fs::copy(&format!("{}-latest.csv", prefix), &format!("{}-real-best.csv", prefix));
                        best_real_len = main_tour.get_real_len();
                    }

                    our_tour_hash = cur_hash;
                    println!("done saving");
                }
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
