extern crate updater;
extern crate rand;
#[macro_use]
extern crate structopt;
extern crate rayon;

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
use rayon::prelude::*;
use std::time::{Duration, Instant};

/// The logistic aka sigmoid function.
#[inline]
pub fn sigmoid(f: f64) -> f64 {
    use std::f64::consts::E;
    1.0 / (1.0 + E.powf(-f))
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

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "t", long = "temp", default_value = "0.03")]
    temp: f64,

    #[structopt(short = "pt", long = "penalty-threshold", default_value = "0.0")]
    penalty_threshold: f64,

    #[structopt(short = "p", long = "penalty", default_value = "0.1")]
    penalty: f64,

    #[structopt(short = "n", long = "n-threads", default_value = "2")]
    n_threads: usize,

    #[structopt(short = "l", long = "load")]
    load_from: String,

    #[structopt(short = "s", long = "save-to")]
    save_to: String,

    #[structopt(short = "c", long = "cand-limit", default_value = "10")]
    cand_limit: usize,

    #[structopt(short = "bl", long = "base-limit", default_value = "3.0")]
    base_limit: f64,

    #[structopt(short = "p", long = "population", default_value = "20")]
    population: usize,

    #[structopt(short = "f", long = "fanout", default_value = "3")]
    fanout: usize,

    #[structopt(short = "i", long = "iter-step", default_value = "200000")]
    iter_step: usize,

    #[structopt(short = "i", long = "iter-limit", default_value = "20")]
    iter_limit: u64,
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    unsafe {
        penalty_config.base_penalty = opt.penalty;

        if opt.penalty_threshold > 0.0 {
            let threshold = opt.penalty_threshold;

            penalty_config.penalty_lambda = Some(
                Box::new(move |len, pos| {
                    sigmoid(((len / (threshold + 1e-10)) - 1.0) * 5.0)
                })
            );
        }
    }

    /*let f = File::open(opt.schedule_file).expect("file not found");
    let file = BufReader::new(&f);
    let penalty_schedule = file.lines().map(|x| {
        let line = x.unwrap();
        println!("line {}", line);
        let mut parts = line.split(' ');
        let p1 = parts.next().unwrap().parse::<f64>().unwrap();
        let p2 = parts.next().unwrap().parse::<f64>().unwrap();
        (p1, p2)
    }).collect::<Vec<_>>();*/


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

    let mut population = (0..opt.population).map(|_| base_tour.clone()).collect::<Vec<_>>();
    let iter_step = opt.iter_step;
    let temp = opt.temp;
    let base_limit = opt.base_limit;

    /*for (penalty, penalty_threshold) in penalty_schedule {
        unsafe {
            penalty_config.base_penalty = penalty;

            if penalty_threshold > 0.0 {
                let threshold = penalty_threshold;

                penalty_config.penalty_lambda = Some(
                    Box::new(move |len, pos| {
                        sigmoid(((len / (threshold + 1e-10)) - 1.0) * 5.0)
                    })
                );
            }
        }*/
    //for threshold in vec![200.0, 180.0, 150.0, 125.0, 100.0, 70.0, 50.0, 40.0, 30.0, 20.0, 15.0, 12.0, 10.0, 9.5, 9.0, 8.5,8.0,7.5,7.0,6.5,6.0,5.5,5.0,4.5,4.0,3.5,3.0,2.5,2.0,1.5,1.0,0.5, 0.00000001] {
    for penalty_base in 1..23 {
        let penalty =  if penalty_base < 3 {
            ((penalty_base) as f64) * 0.001
        } else {
            ((penalty_base - 2) as f64) * 0.005
        };
        unsafe {
            penalty_config.base_penalty = penalty;
            //let threshold = penalty_threshold;

            /*penalty_config.penalty_lambda = Some(
                Box::new(move |len, pos| {
                    sigmoid(((len / (threshold + 1e-10)) - 1.0) * 5.0)
                })
            );*/
        }
        let start = Instant::now();
        population = population.into_iter().map(|x| x.recompute()).collect();
        population.sort_unstable_by(|t, t2| t.get_len().partial_cmp(&t2.get_len()).unwrap());

        println!("starting penalty {} {} {}", penalty, population[0].get_len(), population[0].get_real_len());

        loop {
            let mut new_population = Vec::new();
            for t in &population {
                for f in 0..opt.fanout {
                    new_population.push(t.clone());
                }
            }
            new_population.par_iter_mut().enumerate().for_each(|(i, mut tour)| {
                let prefix = format!("{}-", i);
                let mut added_v = vec!();
                let mut removed_v = vec!();
                let mut cand_buf = vec!();
                for iter in 0..iter_step {
                    if let Some((new_tour, pr)) = do_opt(&mut tour, &candidates_w, temp, base_limit, &prefix, &mut added_v, &mut removed_v, &mut cand_buf) {
                        *tour = new_tour;
                    }
                }
            });

            new_population.sort_unstable_by(|t, t2| t.get_len().partial_cmp(&t2.get_len()).unwrap());
            new_population.dedup_by(|a, b| a.get_path() == b.get_path());

            println!("np {} {} {}", new_population.len(), new_population[0].get_len(), new_population[0].get_real_len() );

            new_population.truncate(opt.population);

            population = new_population;
            population[0].save(&format!("{}-{}.csv", opt.save_to, penalty));
            if start.elapsed().as_secs() > opt.iter_limit {
                break;
            }
        }
    }


    /*let tour_hash = Arc::new(AtomicUsize::new(tour.lock().unwrap().hash()));
    let cand_tour = Arc::new(Mutex::new(base_tour.clone()));

    let mut handles = vec![];
    let temp = opt.temp;
    println!("temp {}", temp);

    for thread_id in opt.n_heavy_threads..opt.n_threads + opt.n_heavy_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let cand_tour_mutex = Arc::clone(&cand_tour);
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
            let mut rng = rand::thread_rng();
            loop {
                if let Some((new_tour, pr)) = do_opt(&mut our_tour, &our_candidates, temp, base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf) {
                    {
                        let mut cand_tour = cand_tour_mutex.lock().unwrap();
                        if new_tour.get_len() < cand_tour.get_len() {
                            *cand_tour = new_tour.clone();
                        }
                        if rng.gen_range(0, 10) == 0 {
                            println!("full accept {} {}", cand_tour.get_len(), cand_tour.get_real_len());
                            let mut main_tour = main_tour_mutex.lock().unwrap();
                            *main_tour = cand_tour.clone();
                            main_tour_hash.store(main_tour.hash(), Ordering::Relaxed);
                        }
                        /*let mut main_tour = main_tour_mutex.lock().unwrap();
                        if new_tour.get_len() < main_tour.get_len() || (temp > 0.0 && ((main_tour.get_len() - new_tour.get_len()) / temp).exp() > pr) {
                            our_tour = new_tour;
                            our_tour_hash = our_tour.hash();


                            *main_tour = our_tour.clone();
                            main_tour_hash.store(our_tour_hash, Ordering::Relaxed);
                        }*/
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
    }*/
}