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
use std::time;
use std::io::stdout;
use std::hash::Hasher;
use std::hash::Hash;
use std::iter;

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
    println!("merge {} {} {}", &f1, &f2, &f3);
    let cur_pen = format!("{}", penalty_config.base_penalty);
    let ls = format!("{}", penalty_config.length_slope);
    let mb = format!("{}", penalty_config.max_penalty_bonus);

    a.save(&f1);
    b.save(&f2);

    Command::new("./recombinator2").arg(&f1).arg(&f2).arg(&f3).arg(&cur_pen).arg(&ls).arg(&mb).status().expect("recomb failed");

    a.make_new(load_tour(&f3))
}

const n_configs: usize = 4;

const opt_configs: [(f64, f64, f64, usize, f64, usize, usize); n_configs] = [
    (1.0, 0.0, 0.0, 500000, 0.3, 0, 3),     // 0 x    18 24 30 36
    (1.0, 0.0, 0.0, 1000000, 0.3, 0, 3),     // 1 x    18 24 30 36

    (2.5, 0.0, 0.0, 800000, 0.0, 0, 3),
    (3.0, 0.0, 0.0, 800000, 0.0, 0, 3),
];

/*fn do_opt2p(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], prefix: &str, base_limit: f64, thread_id: usize) -> Option<Tour> {
    let (bp, ls, lms, iters, temp, min_k, tabus) = opt_configs[thread_id % n_configs];

    let mut rng = rand::thread_rng();

    //let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty, length_slope: 0.01, length_min_slope: 10.0 });
    //let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty * bp, length_slope: ls, length_min_slope: lms, ..Default::default() });
    let mut penalty_config = tour.get_penalty_config();
    penalty_config.base_penalty *= bp;
    let mut cur_tour = tour.change_penalty(penalty_config);

    let mut cc = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = Vec::new();

    let mut tabu = HashSet::new();

    for i in 0..iters {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, pi,temp, base_limit, "heavy-start-", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), min_k) {
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

    println!("go {} {} {} ", thread_id % n_configs, start_len, actual_len);


    let mut last = 0;
    let mut fouls = 0;
    let mut cand_buf = vec!();
    let mut found_opts = 0;
    let no_tabu = HashSet::new();
    for i in 0..1_000_000_000 {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, pi,0.0, base_limit, "heavy-", &mut added_v, &mut removed_v, &mut cand_buf, if found_opts >= tabus {
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
        if i > 1_000_000 {
            break;
        }
        if actual_len < start_len {
            break;
        }
    }

    let mut perm = (1..tour.get_path().len()-1).collect::<Vec<_>>();
    perm.shuffle(&mut rng);
    loop {
        if let Some(new_tour) = do_opt_all(&mut cur_tour, candidates, pi, base_limit, "heavyb-", &mut added_v, &mut removed_v, &mut cand_buf, perm[cc % perm.len()]) {
            cur_tour = new_tour;
            last = cc;
        }
        cc += 1;
        if cc - last > perm.len() {
            break;
        }
        if cc % 10000 == 0 {
            println!("ccc {}", cc);
        }
    }

    cur_tour = merge(&cur_tour, &tour, prefix, tour.get_penalty_config());
    actual_len = cur_tour.get_len();

    println!("after merge {} {}", actual_len, start_len);

    Some(cur_tour)
}*/

const n_local_configs: usize = 3;

const opt_local_configs: [(bool, f64, usize, f64, usize, f64, usize, usize); n_local_configs] = [
    //(true,  0.1, 5_000, 0.5, 100_000, 0.8, 250_000,1_000_000),
    //(false, 0.1, 250_000, 0.5, 1_000_000, 0.8, 1_000_000,1_000_000),
    //(false, 0.1, 250_000, 0.5, 3_000_000, 0.8, 2_000_000,2_000_000),
    //(false, 0.1, 125_000, 0.5, 1_000_000, 0.8, 1_000_000,1_000_000),
    //(false, 0.1, 125_000, 0.5, 3_000_000, 0.8, 2_000_000,2_000_000),
    //(false, 0.1, 500_000, 0.5, 2_000_000, 0.8, 2_000_000,2_000_000),
    //(false, 0.1, 500_000, 0.5, 3_000_000, 0.8, 2_000_000,2_000_000),
    (false, 3.0, 100_000, 2.0, 200_000, 1.5, 500_000,1_000_000),
    (false, 2.0, 100_000, 1.5, 200_000, 1.2, 500_000,1_000_000),
    (false, 5.0, 100_000, 3.0, 200_000, 1.5, 500_000,1_000_000),
    /*    (2.0, 250_000, 1.5, 1_000_000, 1.2, 5_000_000,5_000_000),
        (2.0, 250_000, 1.5, 2_000_000, 1.2, 5_000_000,5_000_000),
        (2.0, 500_000, 1.5, 1_000_000, 1.2, 5_000_000,5_000_000),
        (2.0, 500_000, 1.5, 2_000_000, 1.2, 5_000_000,5_000_000),
        (2.0, 750_000, 1.5, 1_000_000, 1.2, 5_000_000,5_000_000),
        (2.0, 750_000, 1.5, 2_000_000, 1.2, 5_000_000,5_000_000),*/
];

fn do_opt_break_local(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], prefix: &str, base_limit: f64, thread_id: usize) -> Option<Tour> {
    let (alter_hash, first_penalty, first_iters, second_penalty, second_iters, third_penalty, third_iters, final_iters) = opt_local_configs[thread_id % n_local_configs];

    /*if alter_hash {
        return do_opt_alter_hash(tour, candidates, pi, prefix, base_limit, thread_id);
    }*/

    let mut rng = rand::thread_rng();
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = Vec::new();

    let mut x_min = rng.gen_range(0.0, 4700.0);
    let mut x_max = x_min + 1000.0;
    let mut y_min = rng.gen_range(0.0, 2700.0);
    let mut y_max = y_min + 1000.0;

    let good_nodes = tour.nodes.iter().enumerate().filter_map(|(i, &(x, y))| {
        if x > x_min && x < x_max && y > y_min && y < y_max {
            Some(i)
        } else {
            None
        }
    }).collect::<Vec<_>>();


    //let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty, length_slope: 0.01, length_min_slope: 10.0 });
    let mut cur_tour = tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty * first_penalty, ..tour.get_penalty_config() });
    let prefix1 = format!("break-{}({})-{}", thread_id, thread_id % n_local_configs, first_penalty);
    for i in 0..first_iters {
        let start_vertex = *good_nodes.choose(&mut rng).unwrap();
        let start_vertex_pos = cur_tour.get_inv()[start_vertex];
        if start_vertex == 0 || start_vertex_pos >= cur_tour.get_path().len() - 1 {
            continue;
        }
        let start_vertex2 = cur_tour.get_path()[start_vertex_pos+1];
        if let Some((new_tour, _)) = do_opt_start(&mut cur_tour, candidates, pi, 0.0, base_limit, &prefix1, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0, start_vertex, start_vertex2) {
            cur_tour = new_tour;
        }
    }

    x_min -= 50.0;
    x_max += 50.0;
    y_min -= 50.0;
    y_max += 50.0;

    let good_nodes = tour.nodes.iter().enumerate().filter_map(|(i, &(x, y))| {
        if x > x_min && x < x_max && y > y_min && y < y_max {
            Some(i)
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let prefix2 = format!("break-{}({})-{}", thread_id, thread_id % n_local_configs, second_penalty);
    let mut cur_tour = cur_tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty * second_penalty, ..tour.get_penalty_config() });
    for i in 0..second_iters {
        let start_vertex = *good_nodes.choose(&mut rng).unwrap();
        let start_vertex_pos = cur_tour.get_inv()[start_vertex];
        if start_vertex == 0 || start_vertex_pos >= cur_tour.get_path().len() - 1 {
            continue;
        }
        let start_vertex2 = cur_tour.get_path()[start_vertex_pos+1];
        if let Some((new_tour, _)) = do_opt_start(&mut cur_tour, candidates, pi, 0.0, base_limit, &prefix2, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0, start_vertex, start_vertex2) {
            cur_tour = new_tour;
        }
    }

    x_min -= 50.0;
    x_max += 50.0;
    y_min -= 50.0;
    y_max += 50.0;

    let good_nodes = tour.nodes.iter().enumerate().filter_map(|(i, &(x, y))| {
        if x > x_min && x < x_max && y > y_min && y < y_max {
            Some(i)
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let prefix3 = format!("break-{}({})-{}", thread_id, thread_id % n_local_configs, third_penalty);
    let mut cur_tour = cur_tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty * third_penalty, ..tour.get_penalty_config() });
    for i in 0..third_iters {
        let start_vertex = *good_nodes.choose(&mut rng).unwrap();
        let start_vertex_pos = cur_tour.get_inv()[start_vertex];
        if start_vertex == 0 || start_vertex_pos >= cur_tour.get_path().len() - 1 {
            continue;
        }
        let start_vertex2 = cur_tour.get_path()[start_vertex_pos+1];
        if let Some((new_tour, _)) = do_opt_start(&mut cur_tour, candidates, pi, 0.0, base_limit, &prefix3, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0, start_vertex, start_vertex2) {
            cur_tour = new_tour;
        }
    }

    x_min -= 50.0;
    x_max += 50.0;
    y_min -= 50.0;
    y_max += 50.0;

    let good_nodes = tour.nodes.iter().enumerate().filter_map(|(i, &(x, y))| {
        if x > x_min && x < x_max && y > y_min && y < y_max {
            Some(i)
        } else {
            None
        }
    }).collect::<Vec<_>>();

    let prefix4 = format!("break-{}({})-{}", thread_id, thread_id % n_local_configs, 1.0);
    let mut cur_tour = cur_tour.change_penalty(PenaltyConfig { base_penalty: tour.get_penalty_config().base_penalty, ..tour.get_penalty_config() });
    for i in 0..final_iters {
        let start_vertex = *good_nodes.choose(&mut rng).unwrap();
        let start_vertex_pos = cur_tour.get_inv()[start_vertex];
        if start_vertex == 0 || start_vertex_pos >= cur_tour.get_path().len() - 1 {
            continue;
        }
        let start_vertex2 = cur_tour.get_path()[start_vertex_pos+1];
        if let Some((new_tour, _)) = do_opt_start(&mut cur_tour, candidates, pi, 0.0, base_limit, &prefix4, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0, start_vertex, start_vertex2) {
            cur_tour = new_tour;
        }
    }
    let mut last = 0usize;
    let mut cc = 0usize;
    let mut perm = good_nodes.clone();
    let prefix4b = format!("break-{}({})-{}b", thread_id, thread_id % n_local_configs, 1.0);
    println!("break-{} go fin {}", thread_id, perm.len());
    perm.shuffle(&mut rng);
    loop {
        let sp = cur_tour.get_inv()[perm[cc % perm.len()]];
        if sp >= cur_tour.get_path().len() - 1 || sp <= 1 {
            cc += 1;
            continue;
        }
        if let Some(new_tour) = do_opt_all(&mut cur_tour, candidates, pi, base_limit, &prefix4b, &mut added_v, &mut removed_v, &mut cand_buf, sp) {
            cur_tour = new_tour;
            last = cc;
        }
        cc += 1;
        if cc - last > perm.len() {
            break;
        }
        if cc % 1000 == 0 {
            println!("ccc {} {} {}", cc, cc - last, perm.len());
        }
    }

    cur_tour = merge(&cur_tour, &tour, prefix, tour.get_penalty_config());
    let actual_len = cur_tour.get_len();

    println!("after merge {} {}", actual_len, tour.get_len());

    Some(cur_tour)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kopt")]
struct Config {
    #[structopt(short = "p", long = "penalty", default_value = "0.1")]
    penalty: f64,

    #[structopt(short = "bc", long = "big-cutoff", default_value = "50.0")]
    big_cutoff: f64,


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

    #[structopt(short = "sl", long = "state-limit", default_value = "2000")]
    state_limit: usize,

    #[structopt(short = "n", long = "n-threads", default_value = "2")]
    n_threads: usize,

    #[structopt(short = "nh", long = "n-heavy-threads", default_value = "2")]
    n_heavy_threads: usize,

    #[structopt(short = "nm", long = "n-merge-threads", default_value = "2")]
    n_merge_threads: usize,

    #[structopt(short = "nho", long = "n-hot-threads", default_value = "2")]
    n_hot_threads: usize,

    #[structopt(long = "min-coverage", default_value = "0")]
    min_coverage: usize,
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
        println!("top lens {:?}", &lens[..10.min(lens.len())]);
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
            (-(tour.get_len() - best_key.1)/50.0).exp()
        }).unwrap()).clone()
    }

    pub fn best_tour(&self, k: usize) -> Tour {
        let mut lens = self.data.values().collect::<Vec<_>>();
        lens.sort_by(|a, b| a.get_len().partial_cmp(&b.get_len()).unwrap());

        lens[k.min(lens.len()-1)].clone()
    }

    pub fn save(&self, save_dir: &str) {
        self.data.iter().for_each(|(k, v)| {
            let path = format!("{}/{}-{:.2}.csv", save_dir, k, v.get_len());
            v.save(&path);
        })
    }
}

fn main() {
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());
    let pi = load_pi(nodes.len());

    let mut penalty_config: PenaltyConfig = Default::default();
    penalty_config.base_penalty = opt.penalty;
    penalty_config.big_cutoff = opt.big_cutoff;
    penalty_config.min_coverage = opt.min_coverage;

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
        let greedy = thread_id < opt.n_threads / 2;
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
                let mut tour = if greedy {
                    states_mutex.lock().unwrap().best_tour(thread_id)
                } else {
                    states_mutex.lock().unwrap().pick_tour()
                };
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
                }
                println!("thread {} finished {} {}", thread_id, tour.get_len(), tour.get_real_len());
                states_mutex.lock().unwrap().add_tour(tour);
            }
        });
        handles.push(handle);
    }

    for thread_id in 0..opt.n_hot_threads {
        let greedy = thread_id < opt.n_hot_threads / 2;
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
                let mut tour = if greedy {
                    states_mutex.lock().unwrap().best_tour(thread_id)
                } else {
                    states_mutex.lock().unwrap().pick_tour()
                };
                let mut start_hash = tour.penalty_hash;

                let mut cc = 0usize;
                let mut last = 0usize;
                println!("thread {}ho starting {} {}", thread_id, tour.get_len(), tour.get_real_len());
                for _ in 0..200_000 {
                    if let Some((new_tour, _pr)) = do_opt(&mut tour, &our_candidates, &our_pi,1.0, base_limit, &prefixacc, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
                        if new_tour.penalty_hash != start_hash {
                            states_mutex.lock().unwrap().add_tour(new_tour.clone());
                        }
                    }
                }
                println!("thread {}ho finished {} {}", thread_id, tour.get_len(), tour.get_real_len());
            }
        });
        handles.push(handle);
    }

    for thread_id in 0..opt.n_heavy_threads {
        let greedy = thread_id < opt.n_heavy_threads / 2;
        let states_mutex = Arc::clone(&states);
        let our_candidates = candidates_w.clone();
        let prefix = format!("{}/tmp-{}", opt.save_to.clone(), thread_id);
        let prefixacc = format!("{}h", thread_id);
        let base_limit = opt.base_limit;
        let our_pi = pi.clone();
        let min_coverage = opt.min_coverage;
        let handle = thread::spawn(move || {
            //thread::sleep(time::Duration::new(60, 0));
            /*loop {
                let mut tour = if greedy {
                    states_mutex.lock().unwrap().best_tour(thread_id)
                } else {
                    states_mutex.lock().unwrap().pick_tour()
                };

                let mut cc = 0usize;
                let mut last = 0usize;
                println!("thread {}h starting {} {}", thread_id, tour.get_len(), tour.get_real_len());
                if let Some(new_tour) = do_opt_break_local(&mut tour, &our_candidates, &our_pi,&prefix, base_limit, thread_id) {
                    let len = new_tour.get_len();
                    let real_len = new_tour.get_real_len();
                    states_mutex.lock().unwrap().add_tour(new_tour);
                    println!("thread {}h finished {} {}", thread_id, len, real_len);
                }

            }*/
            loop {
                let mut tour = if greedy {
                    states_mutex.lock().unwrap().best_tour(thread_id)
                } else {
                    states_mutex.lock().unwrap().pick_tour()
                };

                let mut cc = 0usize;
                let mut last = 0usize;
                println!("thread {}h starting {} {}", thread_id, tour.get_len(), tour.get_real_len());
                let moves = list_moves(&mut tour, &our_candidates, &our_pi, base_limit);
                let mut scores = moves.iter().enumerate().filter_map(|(i, m)| {
                    let r = m.range(&tour);
                    if r.1 - r.0 < min_coverage {
                        None
                    } else {
                        tour.test_changes_fast(&m.added, &m.removed).map(|l| (l, i))
                    }
                }).collect::<Vec<_>>();

                scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
                for &(_, i) in &scores[..50] {
                    let (_, p) = tour.test_changes(&moves[i].added, &moves[i].removed).unwrap();
                    let new_tour = tour.make_new(p);
                    states_mutex.lock().unwrap().add_tour(new_tour);
                }
                println!("thread {}h done", thread_id);
            }
        });
        handles.push(handle);
    }

    for thread_id in 0..opt.n_merge_threads {
        let states_mutex = Arc::clone(&states);
        let our_candidates = candidates_w.clone();
        let prefix = format!("{}/tmp-{}m", opt.save_to.clone(), thread_id);
        let prefixacc = format!("{}h", thread_id);
        let base_limit = opt.base_limit;
        let our_pi = pi.clone();
        let handle = thread::spawn(move || {
            loop {
                let tour = states_mutex.lock().unwrap().pick_tour();
                let tour2 = states_mutex.lock().unwrap().pick_tour();


                println!("thread {}m starting {} {}", thread_id, tour.get_len(), tour.get_real_len());
                let tourm = merge(&tour, &tour2, &prefix, tour.get_penalty_config());
                let len = tourm.get_len();
                let real_len = tourm.get_real_len();
                states_mutex.lock().unwrap().add_tour(tourm);
                println!("thread {}m finished {} {}", thread_id, len, real_len);
            }
        });
        handles.push(handle);
    }

    {
        let states_mutex = Arc::clone(&states);
        let save_to = opt.save_to.clone();
        let handle = thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_millis(60_000));
                println!("saving");
                states.lock().unwrap().save(&save_to);
                println!("saving done");
            }
        });
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn list_moves(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64) -> Vec<Move> {
    let mut ret1 = Vec::new();

    for pos in 1..tour.get_path().len()-1 {
        let v1 = tour.get_path()[pos];
        let v2 = tour.get_path()[pos+1];
        let mut removed = vec![(v1, v2)];
        let removed_sum = dist_pi(pi, &tour.nodes, v1, v2);
        ret1.extend(list_all_inner(tour, candidates, pi, base_limit, &mut Vec::new(), &mut removed, v1, v2, 2, 0.0, removed_sum));
        if pos % 10000 == 0 {
            println!("p {} rs {}", pos, ret1.len());
        }
    }

    let ret_set = ret1.iter().map(|x| x.normalize()).collect::<HashSet<_>>();
    let ret = ret_set.into_iter().collect::<Vec<_>>();

    println!("2opt {}", ret.iter().map(|m| if m.added.len() == 2 { 1 } else { 0 }).sum::<usize>());
    println!("3opt {}", ret.iter().map(|m| if m.added.len() == 3 { 1 } else { 0 }).sum::<usize>());
    println!("4opt {}", ret.iter().map(|m| if m.added.len() == 4 { 1 } else { 0 }).sum::<usize>());

    ret
}

fn list_patch(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, mut all_cycle_parts: Vec<Vec<(usize, usize)>>, mut added_sum: f64, mut removed_sum: f64) -> Vec<Move> {
    let mut ret = Vec::new();
    if added_sum - removed_sum > base_limit {
        return ret;
    }

    let mut cycle_parts = all_cycle_parts.into_iter().next().unwrap();

    cycle_parts.iter_mut().for_each(|p| {
        *p = (p.0.min(p.1), p.0.max(p.1))
    });

    for cp in &cycle_parts {
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
                            ret.push(Move{ added: added.clone(), removed: removed.clone(), gain: added_sum - removed_sum});
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

    ret
}

fn list_all_inner(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], base_limit: f64, added: &mut Vec<(usize, usize)>, removed: &mut Vec<(usize, usize)>, current_vertex: usize, start_vertex2: usize, max_k: usize, mut added_sum: f64, mut removed_sum: f64) -> Vec<Move> {
    let mut ret = Vec::new();
    if removed.len() >= 2 {
        added.push((current_vertex, start_vertex2));
        added_sum += dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
        if added_sum - removed_sum < base_limit {
            //println!("counting");
            let (cycles, cycle_parts) = tour.count_cycles(&added, &removed);
            let left = max_k;
            if cycles > left + 1 && cycles < 1_000_000 {
            } else if added.len() % 5 == 0 && cycles > 1 {
            } else {
                if cycles == 1 {
                    ret.push(Move { added: added.clone(), removed: removed.clone(), gain: added_sum - removed_sum });
                } else if cycles == 2 {
                    ret.extend(list_patch(tour, candidates, pi, base_limit, added, removed, cycle_parts, added_sum, removed_sum));
                }
            }
        }

        added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, start_vertex2);
        added.pop();
    }
    if max_k > 0 {
        let cand_buf = candidates[current_vertex].iter().filter(|&&(c, d)| d + pi[current_vertex] + pi[c] <= removed_sum - added_sum + base_limit).map(|&x| x.0).collect::<Vec<_>>();

        for &next_vertex in cand_buf.iter() {
            if next_vertex == 0 || removed.contains(&(current_vertex, next_vertex)) || removed.contains(&(next_vertex, current_vertex)) ||
                added.contains(&(current_vertex, next_vertex)) || added.contains(&(next_vertex, current_vertex)) {
                continue;
            }

            added_sum += dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
            added.push((current_vertex, next_vertex));

            let current_cands = tour.neighbours(next_vertex);
            if current_cands[0] != 0 {
                let current_vertex2 = current_cands[0];
                if !removed.contains(&(current_vertex2, next_vertex)) && !removed.contains(&(next_vertex, current_vertex2)) &&
                    !added.contains(&(current_vertex2, next_vertex)) && !added.contains(&(next_vertex, current_vertex2)) {
                    removed.push((next_vertex, current_vertex2));
                    removed_sum += dist_pi(&pi, &tour.nodes, current_vertex2, next_vertex);
                    ret.extend(list_all_inner(tour, candidates, pi, base_limit, added, removed, current_vertex2, start_vertex2,max_k - 1, added_sum, removed_sum));
                    removed.pop();
                    removed_sum -= dist_pi(&pi, &tour.nodes, current_vertex2, next_vertex);
                }
            }
            if current_cands[1] != 0 {
                let current_vertex2 = current_cands[1];
                if !removed.contains(&(current_vertex2, next_vertex)) && !removed.contains(&(next_vertex, current_vertex2)) &&
                    !added.contains(&(current_vertex2, next_vertex)) && !added.contains(&(next_vertex, current_vertex2)) {
                    removed.push((next_vertex, current_vertex2));
                    removed_sum += dist_pi(&pi, &tour.nodes, current_vertex2, next_vertex);
                    ret.extend(list_all_inner(tour, candidates, pi, base_limit, added, removed, current_vertex2, start_vertex2,max_k - 1, added_sum, removed_sum));
                    removed.pop();
                    removed_sum -= dist_pi(&pi, &tour.nodes, current_vertex2, next_vertex);
                }
            }

            added.pop();
            added_sum -= dist_pi(&pi, &tour.nodes, current_vertex, next_vertex);
        }
    }

    ret
}

#[derive(Debug, Clone)]
struct Move {
    added: Vec<(usize, usize)>,
    removed: Vec<(usize, usize)>,
    gain: f64
}

impl Move {
    fn range(&self, tour: &Tour) -> (usize, usize) {
        let mut removed_inds = self.removed.iter().map(|x| iter::once(tour.get_inv()[x.0]).chain(iter::once(tour.get_inv()[x.1]))).flatten().collect::<Vec<_>>();
        let min_removed = *removed_inds.iter().min().unwrap();
        let max_removed = *removed_inds.iter().max().unwrap();
        (min_removed, max_removed)
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Move) -> bool {
        self.added == other.added && self.removed == other.removed
    }
}

impl Eq for Move {}

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.added.hash(state);
        self.removed.hash(state);
    }
}

fn normalize_vec(x: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut ret = x.iter().map(|&(a, b)| { (a.min(b), a.max(b)) }).collect::<Vec<_>>();

    ret.sort();

    ret
}

impl Move {
    fn normalize(&self) -> Move {
        Move { added: normalize_vec(&self.added), removed: normalize_vec(&self.removed), gain: self.gain }
    }
}