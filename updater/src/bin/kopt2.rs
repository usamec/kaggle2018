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

fn do_opt_inner(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], min_ind: usize, max_ind: usize) -> Option<Tour> {
    let mut rng = rand::thread_rng();
    let start_path_pos = rng.gen_range(min_ind+1, max_ind);
    let start_vertex = tour.get_path()[start_path_pos];
    let start_vertex2 = tour.get_path()[start_path_pos+1];

    let mut removed = Vec::new();
    let mut added = Vec::new();
    removed.push((start_vertex, start_vertex2));

    let mut current_vertex = start_vertex;
    let mut removed_sum = dist(tour.nodes[start_vertex], tour.nodes[start_vertex2]);
    let mut added_sum = 0.0;
    for i in 0..*[2,3,4].choose(&mut rng).unwrap() {
        let mut next_vertex = 0;
        for _ in 0..100 {
            next_vertex = candidates[current_vertex].choose_weighted(&mut rng, |x| x.1).unwrap().0;
            if next_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)) {
                if tour.get_inv()[next_vertex] < max_ind &&  tour.get_inv()[next_vertex] > min_ind {
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

        if added_sum - removed_sum > 5.0 {
            return None
        }


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

    let test_fast = tour.test_changes_fast(&added, &removed);
    if let Some(len) = test_fast {
        if len < tour.get_len() {
            let (res, p) = tour.test_changes(&added, &removed).unwrap();
            Some(tour.make_new(p, ))
        } else {
            None
        }
    } else {
        None
    }
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
    for i in 0..1_000_000_000 {
        if let Some(new_tour) = do_opt_inner(&mut cur_tour, candidates, min_removed-3, max_removed+3) {
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
        if i - last > 5_000_000 {
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

fn do_opt(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], temp: f64) -> Option<(Tour, f64)> {
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
    for i in 0..*[2,3,4].choose_weighted(&mut rng, |x| x*x).unwrap() {
        let mut next_vertex = 0;
        loop {
            next_vertex = candidates[current_vertex].choose_weighted(&mut rng, |x| x.1).unwrap().0;
            if next_vertex != 0 {
                break;
            }
        }
        added_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
        added.push((current_vertex, next_vertex));

        if added_sum - removed_sum > 5.0 {
            return None
        }


        loop {
            current_vertex = tour.rand_neighbour(next_vertex);
            if current_vertex != 0 {
                break;
            }
        }

        removed_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
        removed.push((next_vertex, current_vertex));
    }

    added.push((current_vertex, start_vertex2));
    added_sum += dist(tour.nodes[current_vertex], tour.nodes[start_vertex2]);

    /*if added_sum - removed_sum > 10.0 {
        return None
    }*/

    /*if rng.gen_range(0, 1) == 0 {
        println!("diff {}", added_sum - removed_sum);
    }*/

    //println!("{:?}", removed);
    //println!("{:?}", added);

    let test_fast = tour.test_changes_fast(&added, &removed);
    if let Some(len) = test_fast {
        let pr = rng.gen::<f64>();
        if len < tour.get_len() || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > pr) {
            let (res, p) = tour.test_changes(&added, &removed).unwrap();
            let new_tour = tour.make_new(p, );
            println!("accept {} real {}, added len {} added - removed {}", new_tour.get_len(), new_tour.get_real_len(), added.len(), added_sum - removed_sum);
            stdout().flush();
            Some((new_tour, pr))
        } else {
            None
        }
    } else {
        None
    }
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
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    unsafe {
        penalty_config.base_penalty = opt.penalty;

        if opt.penalty_threshold > 0.0 {
            let penalty_threshold = opt.penalty_threshold;

            let threshold = penalty_threshold;

            penalty_config.penalty_lambda = Some(
                Box::new(move |len, pos| {
                    if len > threshold {
                        1.0
                    } else {
                        len / threshold
                    }
                })
            );
        }
    }


    let primes = Arc::new(get_primes(nodes.len()));
    let tour = Arc::new(Mutex::new(Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone())));
    let candidates = load_candidates(opt.cand_limit);
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().enumerate().map(|(j, &c)| {
            let d = dist(nodes[i], nodes[c]);
            (c, 1.0 / d)
        }).collect::<Vec<_>>()
        //cc.iter().map(|&c| (c, 1.0)).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.lock().unwrap().get_path().len(), candidates.len());
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
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some((new_tour, pr)) = do_opt(&mut our_tour, &our_candidates, temp) {
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
