extern crate mega_opt;
extern crate rand;
#[macro_use]
extern crate structopt;
extern crate chrono;

use mega_opt::*;
use rand::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;

use std::sync::{Mutex, Arc};
use std::thread;
use structopt::StructOpt;
use std::fs;
use std::process::Command;

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

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
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
    cand_file: String,

    #[structopt(short = "sl", long = "state-limit", default_value = "20")]
    state_limit: usize,

    #[structopt(short = "n", long = "n-threads", default_value = "2")]
    n_threads: usize,

}

struct States {
    data: HashMap<usize, Tour>,
    limit: usize,
    pub nodes: Arc<Vec<(f64,f64)>>,
    primes: Arc<Vec<bool>>,
    penalty_config: PenaltyConfig,
}

impl States {
    pub fn load_from_dir(dir: &str, limit: usize, nodes: Arc<Vec<(f64, f64)>>, primes: Arc<Vec<bool>>, penalty_config: PenaltyConfig) -> States {
        let mut st = States {data: HashMap::new(), limit: limit, nodes: nodes.clone(), primes: primes.clone(), penalty_config: penalty_config };

        fs::read_dir(dir).unwrap().for_each(|x| {
            let tour = Tour::new(load_tour(&x.unwrap().path().to_str().unwrap()), nodes.clone(), primes.clone(), penalty_config);
            st.add_tour(tour);
        });

        st
    }

    pub fn add_tour(&mut self, tour: Tour) {
        if self.data.contains_key(&tour.penalty_hash) {
            if self.data.get(&tour.penalty_hash).unwrap().get_len() < tour.get_len() {
                return;
            }
        }
        println!("adding tour with hash {} and len {} {}", tour.penalty_hash, tour.get_len(), tour.get_real_len());

        self.data.insert(tour.penalty_hash, tour);

        if self.data.len() > self.limit {
            let worst_key = self.data.iter().fold((0usize, 0.0), |(hash, len), (&cur_hash, cur_tour)|{
                if cur_tour.get_len() > len {
                    (cur_hash, cur_tour.get_len())
                } else {
                    (hash, len)
                }
            });

            let deleted = self.data.remove(&worst_key.0).unwrap();
            println!("removing tour with hash {} and len {} {}", deleted.penalty_hash, deleted.get_len(), deleted.get_real_len());
        }

        let mut lens = self.data.values().map(|x| x.get_len()).collect::<Vec<_>>();
        lens.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("lens {:?}", lens);
    }

    pub fn pick_tour(&self) -> Tour {
        let best_key = self.data.iter().fold((0usize, 1e20), |(hash, len), (&cur_hash, cur_tour)|{
            if cur_tour.get_len() < len {
                (cur_hash, cur_tour.get_len())
            } else {
                (hash, len)
            }
        });
        (*self.data.values().collect::<Vec<_>>().choose_weighted(&mut rand::thread_rng(), |tour| {
            (-(tour.get_len() - best_key.1)/100.0).exp()
        }).unwrap()).clone()
    }
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());
    let pi = load_pi(nodes.len());

    let mut penalty_config: PenaltyConfig = Default::default();
    penalty_config.base_penalty = opt.penalty;

    let primes = Arc::new(get_primes(nodes.len()));

    let candidates = load_candidates2(opt.cand_limit, &opt.cand_file);
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().enumerate().map(|(_j, &c)| {
            let d = dist(nodes[i], nodes[c]);
            (c, d)
        }).collect::<Vec<_>>()
        //cc.iter().map(|&c| (c, 1.0)).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let states = Arc::new(Mutex::new(States::load_from_dir(&opt.load_from, opt.state_limit,nodes.clone(), primes.clone(), penalty_config)));

    let mut handles = vec![];

    for thread_id in 0..opt.n_threads {
        let states_mutex = Arc::clone(&states);
        let our_candidates = candidates_w.clone();
        let prefix = format!("{}-tmp-{}", opt.save_to.clone(), thread_id);
        let prefixacc = format!("{}", thread_id);
        let base_limit = opt.base_limit;
        let our_pi = pi.clone();
        let handle = thread::spawn(move || {
            let mut added_v = vec!();
            let mut removed_v = vec!();
            let mut cand_buf = vec!();
            loop {
                let mut tour = states_mutex.lock().unwrap().pick_tour();
                let mut start_hash = tour.penalty_hash;

                let mut cc = 0usize;
                let mut last = 0usize;
                println!("thread {} starting {} {}", thread_id, tour.get_len(), tour.get_real_len());
                for _ in 0..200_000 {
                    if let Some((new_tour, _pr)) = do_opt(&mut tour, &our_candidates, &our_pi,0.0, base_limit, &prefixacc, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
                        if new_tour.penalty_hash != start_hash {
                            states_mutex.lock().unwrap().add_tour(tour.clone());
                        }
                        tour = new_tour;
                        start_hash = tour.penalty_hash;
                        last = cc;
                    }

                    /*cc += 1;
                    if cc - last > 200_000 {
                        break;
                    }*/

                    /*if cc % 1000000 == 0 {
                        println!("cc {} {} {} {} {}", cc, thread_id, tour.get_len(), tour.get_real_len(), Local::now().format("%Y-%m-%dT%H:%M:%S"));
                    }*/
                }
                println!("thread {} finished {} {}", thread_id, tour.get_len(), tour.get_real_len());
                states_mutex.lock().unwrap().add_tour(tour);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}