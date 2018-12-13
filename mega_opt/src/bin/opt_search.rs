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

fn merge(a: &Tour, b: &Tour, prefix: &str, penalty_config: PenaltyConfig) -> Tour {
    let f1 = format!("{}-1.csv", prefix);
    let f2 = format!("{}-2.csv", prefix);
    let f3 = format!("{}-3.csv", prefix);
    let cur_pen = format!("{}", penalty_config.base_penalty);
    let cur_thres = format!("{}", 0.0);

    a.save(&f1);
    b.save(&f2);

    Command::new("./recombinator").arg(&f1).arg(&f2).arg(&f3).arg(&cur_pen).arg(&cur_thres).status().expect("recomb failed");

    a.make_new(load_tour(&f3))
}

const opt_configs: [(f64, f64, f64, usize, f64, usize, usize); 24] = [
    //(0.1, 0.0, 0.0, 200000, 0.3, 4, 0),     // 0 x
    //(0.1, 0.0, 0.0, 400000, 0.3, 4, 0),     // 1      y
    (0.1, 0.0, 0.0, 1000000, 0.3, 4, 0),     // 0
    (0.1, 0.0, 0.0, 1400000, 0.3, 4, 0),     // 1
    (0.1, 0.0, 0.0, 600000, 0.3, 4, 0),     // 2      y

    //(0.1, 0.0, 0.0, 200000, 0.3, 0, 3),     // 3 x    yyyyy
    (0.1, 0.0, 0.0, 1000000, 0.3, 0, 3),    // 3b
    //(0.1, 0.0, 0.0, 400000, 0.3, 0, 3),     // 4 x    yyyyy
    (0.1, 0.0, 0.0, 1400000, 0.3, 0, 3),    // 4b
    (0.1, 0.0, 0.0, 600000, 0.3, 0, 3),     // 5 x    yyyyyyy

    (0.25, 0.0, 0.0, 200000, 0.0, 4, 0),    // 6      yyy
    (0.25, 0.0, 0.0, 400000, 0.0, 4, 0),    // 7 x
    (0.25, 0.0, 0.0, 600000, 0.0, 4, 0),    // 8 xx   yyyyy

    //(0.25, 0.0, 0.0, 200000, 0.0, 0, 3),    // 9      yy
    //(0.25, 0.0, 0.0, 400000, 0.0, 0, 3),    //10      yy
    (0.25, 0.0, 0.0, 1000000, 0.0, 0, 3),    // 9
    (0.25, 0.0, 0.0, 1400000, 0.0, 0, 3),    //10
    (0.25, 0.0, 0.0, 600000, 0.0, 0, 3),    //11      yyyy

    (0.1, 0.01, 10.0, 200000, 0.0, 4, 0),   //12 x    yy
    (0.1, 0.01, 10.0, 400000, 0.0, 4, 0),   //13      yyyy
    (0.1, 0.01, 10.0, 600000, 0.0, 4, 0),   //14

    //(0.1, 0.01, 10.0, 600000, 0.0, 0, 3),   //15
    //(0.1, 0.01, 10.0, 1000000, 0.0, 0, 3),  //16
    (0.1, 0.01, 10.0, 1800000, 0.0, 0, 3),   //15
    (0.1, 0.01, 10.0, 2200000, 0.0, 0, 3),  //16
    (0.1, 0.01, 10.0, 1400000, 0.0, 0, 3),  //17 x    y

    (0.1, 0.01, 5.0, 200000, 0.0, 4, 0),    //18 xx   yy
    (0.1, 0.01, 5.0, 400000, 0.0, 4, 0),    //19
    (0.1, 0.01, 5.0, 600000, 0.0, 4, 0),    //20      yyyy

    (0.1, 0.01, 5.0, 400000, 0.0, 0, 3),    //21 x    yy
    (0.1, 0.01, 5.0, 800000, 0.0, 0, 3),    //22 xx   yyy
    (0.1, 0.01, 5.0, 1200000, 0.0, 0, 3),   //23 x    y
];

fn do_opt2(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], prefix: &str, base_limit: f64, thread_id: usize) -> Option<Tour> {
    let (bp, ls, lms, iters, temp, min_k, tabus) = opt_configs[thread_id];

    let mut rng = rand::thread_rng();

    //let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty, length_slope: 0.01, length_min_slope: 10.0 });
    let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: bp, length_slope: ls, length_min_slope: lms });

    let mut cc = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = Vec::new();

    let mut tabu = HashSet::new();

    for i in 0..iters {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, temp, base_limit, "heavy-start-", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), min_k) {
            {
                added_v.iter().for_each(|&x| {
                    tabu.insert(x);
                });
                cur_tour = new_tour;
            }
        }
    }

    cur_tour = cur_tour.change_penalty(tour.get_penalty_config());

    let start_len = tour.get_len();
    let mut actual_len = cur_tour.get_len();
    let mut actual_real_len = cur_tour.get_real_len();

    println!("go {} {} {} ", thread_id, start_len, actual_len);


    let mut last = 0;
    let mut fouls = 0;
    let mut cand_buf = vec!();
    let mut found_opts = 0;
    let no_tabu = HashSet::new();
    for i in 0..1_000_000_000 {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, 0.0, base_limit, "heavy-", &mut added_v, &mut removed_v, &mut cand_buf, if found_opts >= tabus {
            &no_tabu
        } else {
            &tabu
        }, 0) {
            if new_tour.get_path() != tour.get_path() {
                if new_tour.get_len() < actual_len {
                    found_opts += 1;
                    actual_len = new_tour.get_len();
                    actual_real_len = new_tour.get_real_len();
                    println!("bet {} {} {}", actual_len, start_len, i - last);
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
        if i - last > 2_000_000 {
            break;
        }
        if actual_len < start_len {
            break;
        }
    }

    cur_tour = merge(&cur_tour, &tour, prefix, tour.get_penalty_config());
    actual_len = cur_tour.get_len();

    println!("after merge {} {}", actual_len, start_len);

    if actual_len < start_len {
        stdout().flush();
        Some(cur_tour)
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

    #[structopt(short = "nh", long = "n-weak-threads", default_value = "1")]
    n_weak_threads: usize,

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

    let penalty_config = Default::default();

    let primes = Arc::new(get_primes(nodes.len()));
    let tour = Arc::new(Mutex::new(Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone(), penalty_config)));
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
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.lock().unwrap().get_path().len(), candidates_w.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour.lock().unwrap().get_path(), &primes, penalty_config));
    println!("{:?}", tour.lock().unwrap().check_nodes_edges().unwrap().0);

    let tour_hash = Arc::new(AtomicUsize::new(tour.lock().unwrap().hash()));

    let mut handles = vec![];
    let temp = opt.temp;
    println!("temp {}", temp);
    for thread_id in 0..opt.n_heavy_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = format!("{}-tmp-{}", opt.save_to.clone(), thread_id);
        let base_limit = opt.base_limit;
        let handle = thread::spawn(move || {
            /*thread::sleep(time::Duration::new(180, 0));*/
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some(new_tour_base) = do_opt2(&mut our_tour, &our_candidates, &prefix, base_limit, thread_id ) {
                    {
                        let main_tour = main_tour_mutex.lock().unwrap().clone();

                        let new_tour = merge(&new_tour_base, &main_tour, &prefix, main_tour.get_penalty_config());
                        if new_tour.get_len() < main_tour_mutex.lock().unwrap().get_len() {
                            println!("acceptxa {} {} real {} {}", thread_id, new_tour.get_len(), new_tour.get_real_len(), Local::now().format("%Y-%m-%dT%H:%M:%S"));
                            our_tour = new_tour;
                            our_tour_hash = our_tour.hash();


                            let mut main_tour = main_tour_mutex.lock().unwrap();
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

    // writer thread
    {
        let save_timestamp = opt.save_timestamp;
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
                        if save_timestamp {
                            let date = Local::now();
                            fs::copy(&format!("{}-latest.csv", prefix), &format!("{}-best-{}.csv", prefix, date.format("%Y-%m-%dT%H:%M:%S")));
                        } else {
                            fs::copy(&format!("{}-latest.csv", prefix), &format!("{}-best.csv", prefix));
                        }
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
