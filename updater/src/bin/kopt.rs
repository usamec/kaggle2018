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

fn do_opt(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], temp: f64) -> Option<Tour> {
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
        if len < tour.get_len() || (temp > 0.0 && ((tour.get_len() - len) / temp).exp() > rng.gen::<f64>()) {
            let (res, p) = tour.test_changes(&added, &removed).unwrap();
            println!("accept {} {} {}", res, added.len(), added_sum - removed_sum);
            stdout().flush();
            Some(tour.make_new(p, ))
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
            let new_len = verify_and_calculate_len(&nodes, &new_tour, &primes);
            println!("new_len {}", new_len);
            stdout().flush();
            if new_len < cur_len {
                println!("better {:?}", new_len);
            }
            Some((new_tour, new_len))
        } else {
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

    #[structopt(short = "p", long = "penalty", default_value = "1.1")]
    penalty: f64,

    #[structopt(short = "m", long = "min-dist-penalty", default_value = "0.0")]
    min_dist_penalty: f64,

    #[structopt(short = "n", long = "n-threads", default_value = "2")]
    n_threads: usize,

    #[structopt(short = "l", long = "load")]
    load_from: String,

    #[structopt(short = "s", long = "save-to")]
    save_to: String,

    #[structopt(short = "c", long = "cand-limit", default_value = "10")]
    cand_limit: usize
}

fn main() {
    let opt = Config::from_args();

    unsafe {
        PENALTY = opt.penalty;
        MIN_DIST_PENALTY = opt.min_dist_penalty;
    }


    let nodes = Arc::new(load_poses());
    let primes = Arc::new(get_primes(nodes.len()));
    let tour = Arc::new(Mutex::new(Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone())));
    let candidates = load_candidates(opt.cand_limit);
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().enumerate().map(|(j, &c)| {
            let d = dist(nodes[i], nodes[c]);
            (c, 1.0 / (d))
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
    for thread_id in 0..opt.n_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some(new_tour) = do_opt(&mut our_tour, &our_candidates, temp) {
                    //println!("new len {}", new_tour.get_len());
                    our_tour = new_tour;
                    our_tour_hash = our_tour.hash();
                    let mut main_tour = main_tour_mutex.lock().unwrap();
                    *main_tour = our_tour.clone();
                    main_tour_hash.store(our_tour_hash, Ordering::Relaxed);
                    our_tour.save(&format!("{}-{}.csv", prefix, thread_id));
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



    for thread_id in 0..opt.n_threads/2 {
        let main_tour_mutex = Arc::clone(&tour);
        let our_nodes = nodes.clone();
        let our_primes = primes.clone();
        let main_tour_hash = Arc::clone(&tour_hash);
        let handle = thread::spawn(move || {
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().get_path().to_vec();
            let mut cur_len = verify_and_calculate_len(&our_nodes, &our_tour, &our_primes);
            let mut rng = rand::thread_rng();
            loop {
                let size = rng.gen_range(40, 41);
                let start = rng.gen_range(1, our_tour.len() - size - 1);
                let maybe_new_tour = local_update(size, start, 0.0, &our_nodes, &our_tour, cur_len, &our_primes, full_optim);
                if let Some((new_tour, new_len)) = maybe_new_tour {
                    println!("better brute {} {}", cur_len, new_len);
                    cur_len = new_len;
                    our_tour = new_tour;
                    let mut main_tour = main_tour_mutex.lock().unwrap();
                    *main_tour = Tour::new(our_tour.clone(), our_nodes.clone(), our_primes.clone());
                    main_tour_hash.store(main_tour.hash(), Ordering::Relaxed);
                }
                cc += 1;
                let main_tour = main_tour_mutex.lock().unwrap();
                our_tour = main_tour.get_path().to_vec();
                cur_len = verify_and_calculate_len(&our_nodes, &our_tour, &our_primes);
                println!("ccb {}", cc);
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }


    /*let mut cc = 0;
    loop {
        if let Some(new_tour) = do_opt(&mut tour, &candidates_w,0.01) {
            //println!("new len {}", new_tour.get_len());
            tour = new_tour;
            tour.save("../outputs/kopt6.csv");
        }
        cc += 1;
        if cc % 1000000 == 0 {
            println!("cc {}", cc);
        }
    }*/
}
