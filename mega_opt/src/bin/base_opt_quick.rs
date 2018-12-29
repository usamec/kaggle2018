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
    let ls = format!("{}", penalty_config.length_slope);
    let mb = format!("{}", penalty_config.max_penalty_bonus);

    a.save(&f1);
    b.save(&f2);

    Command::new("./recombinator2").arg(&f1).arg(&f2).arg(&f3).arg(&cur_pen).arg(&ls).arg(&mb).status().expect("recomb failed");

    a.make_new(load_tour(&f3))
}

/*const opt_configs: [(f64, f64, f64, usize, f64, usize, usize); 6] = [
    (1.0, 0.0, 0.0, 600000, 0.3, 0, 3),     // 0 x    18 24 30 36

    (2.5, 0.0, 0.0, 600000, 0.0, 4, 0),    // 1 xx   19 25 31 37

    (2.5, 0.0, 0.0, 600000, 0.0, 0, 3),    // 2      14 20 26 32

    (1.0, 0.01, 10.0, 400000, 0.0, 4, 0),   // 3      15 21 27 33

    (1.0, 0.01, 5.0, 600000, 0.0, 4, 0),    // 4      16 22 28 34

    (1.0, 0.01, 5.0, 800000, 0.0, 0, 3),    // 5 xx   17 23 29 35
];*/

const n_configs: usize = 2;

const opt_configs: [(f64, f64, f64, usize, f64, usize, usize); n_configs] = [
    //(1.0, 0.0, 0.0, 40000, 0.3, 0, 3),     // 0 x    18 24 30 36

/*    (2.5, 0.0, 0.0, 600000, 0.0, 4, 0),    // 1 xx   19 25 31 37

    (2.5, 0.0, 0.0, 600000, 0.0, 0, 3),    // 2      14 20 26 32

    (1.0, 0.01, 10.0, 400000, 0.0, 4, 0),   // 3      15 21 27 33

    (1.0, 0.01, 5.0, 600000, 0.0, 4, 0),    // 4      16 22 28 34*/

    (2.5, 0.0, 0.0, 40000, 0.0, 0, 3),
    (0.5, 0.0, 0.0, 40000, 0.0, 0, 3),
    /*(1.0, 0.01, 5.0, 800000, 0.0, 0, 3),*/    // 5 xx   17 23 29 35
];

fn do_opt2p(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], prefix: &str, base_limit: f64, thread_id: usize) -> Option<Tour> {
    let (bp, ls, lms, iters, temp, min_k, tabus) = opt_configs[thread_id % n_configs];

    let mut rng = our_rng();

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


    let prefix1 = format!("hstart-{}({})-", thread_id, thread_id % n_configs);

    for i in 0..iters {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, pi,temp, base_limit, &prefix1, &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), min_k) {
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
    let prefix2 = format!("h-{}({})-", thread_id, thread_id % n_configs);
    for i in 0..1_000_000_000 {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, pi,0.0, base_limit, &prefix2, &mut added_v, &mut removed_v, &mut cand_buf, if found_opts >= tabus {
            &no_tabu
        } else {
            &tabu
        }, 0) {
            if new_tour.get_path() != tour.get_path() {
                if new_tour.get_len() < actual_len {
                    found_opts += 1;
                    actual_len = new_tour.get_len();
                    actual_real_len = new_tour.get_real_len();
                    println!("bet {} {} {} {}", thread_id, actual_len, start_len, i - last);
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
        if i - last > 500_000 {
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

const n_local_configs: usize = 1;

const opt_local_configs: [(bool, f64, usize, f64, usize, f64, usize, usize); n_local_configs] = [
    /*(false, 0.1, 50_000, 2.5, 150_000, 0.8, 200_000,200_000),
    (false, 3.0, 50_000, 2.0, 150_000, 1.2, 200_000,200_000),*/
    (false, 3.0, 50_000, 1.5, 10_000, 0.8, 0,200_000),
    /*(false, 3.0, 50_000, 1.5, 0, 0.8, 0,50_000),
    (false, 3.0, 100_000, 1.5, 0, 0.8, 0,100_000),*/
    /*(false, 3.0, 250_000, 0.3, 1_000_000, 1.2, 2_000_000,2_000_000), //0    0
    (false, 0.1, 250_000, 2.5, 1_000_000, 0.8, 2_000_000,2_000_000), //1    1
    (false, 3.0, 350_000, 0.3, 1_300_000, 1.2, 2_300_000,2_300_000), //0    2
    (false, 0.1, 350_000, 2.5, 1_300_000, 0.8, 2_300_000,2_300_000), //1    3
    (false, 0.1, 500_000, 0.5, 3_000_000, 0.8, 2_000_000,2_000_000), //2    4
    //(false, 0.1, 125_000, 0.5, 1_000_000, 0.8, 1_000_000,1_000_000), //3
    //(false, 0.2, 500_000, 0.5, 3_000_000, 0.8, 2_000_000,2_000_000), //4    3
    (false, 6.0, 1000_000, 3.0, 1000_000, 1.5, 2_000_000,3_000_000),   //5  5
    //(false, 6.0, 500_000, 3.0, 500_000, 1.5, 1_000_000,2_000_000),   //6
    (false, 5.0, 500_000, 3.0, 500_000, 1.5, 1_000_000,2_000_000),   //7    6*/
];

fn do_opt_break_local(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], prefix: &str, base_limit: f64, thread_id: usize) -> Option<Tour> {
    let (alter_hash, first_penalty, first_iters, second_penalty, second_iters, third_penalty, third_iters, final_iters) = opt_local_configs[thread_id % n_local_configs];

    /*if alter_hash {
        return do_opt_alter_hash(tour, candidates, pi, prefix, base_limit, thread_id);
    }*/

    let mut rng = our_rng();
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = Vec::new();

    let xrange = rng.gen_range(200.0, 1000.0);
    let yrange = rng.gen_range(200.0, 1000.0);
    let mut x_min = rng.gen_range(0.0, 5200.0 - xrange);
    let mut x_max = x_min + xrange;
    let mut y_min = rng.gen_range(0.0, 3100.0 - yrange);
    let mut y_max = y_min + yrange;

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
            /*if cur_tour.get_len() < tour.get_len() {
                break;
            }*/
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

fn do_opt2b(tour: &mut Tour, candidates: &[Vec<(usize, f64)>], pi: &[f64], prefix: &str, base_limit: f64) -> Option<Tour> {
    let mut rng = our_rng();

    let mut cur_tour = tour.clone();

    let mut cc = 0;
    let mut cand_buf = Vec::new();
    loop {
        let start_path_pos = rng.gen_range(1, cur_tour.get_path().len() - 1);
        let start_vertex = cur_tour.get_path()[start_path_pos];
        let start_vertex2 = cur_tour.get_path()[start_path_pos + 1];

        let mut removed = Vec::new();
        removed.push((start_vertex, start_vertex2));
        let mut added = Vec::new();

        let mut current_vertex = start_vertex;
        let mut removed_sum = dist(cur_tour.nodes[start_vertex], cur_tour.nodes[start_vertex2]);
        let mut added_sum = 0.0;
        let mut good = true;
        for i in 0..8 {
            let mut next_vertex = 0;
            cand_buf.clear();
            cand_buf.extend(candidates[current_vertex].iter().filter(|&&(c, d)| d <= removed_sum - added_sum + 5.0).map(|&x| x.0));
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
                good = false;
                break;
            }
            added_sum += dist(cur_tour.nodes[current_vertex], cur_tour.nodes[next_vertex]);
            added.push((current_vertex, next_vertex));

            if added_sum - removed_sum > 5.0 {
                good = false;
                break;
            }


            current_vertex = 0;
            for _ in 0..100 {
                current_vertex = cur_tour.rand_neighbour(next_vertex);
                if current_vertex != 0 && !removed.contains(&(current_vertex, next_vertex)) && !removed.contains(&(next_vertex, current_vertex)) &&
                    !added.contains(&(current_vertex, next_vertex)) && !added.contains(&(next_vertex, current_vertex)) {
                    break;
                }
                current_vertex = 0;
            }
            if current_vertex == 0 {
                good = false;
                break;
            }

            removed_sum += dist(cur_tour.nodes[current_vertex], cur_tour.nodes[next_vertex]);
            removed.push((next_vertex, current_vertex));
        }
        if !good {
            continue;
        }

        added.push((current_vertex, start_vertex2));
        added_sum += dist(cur_tour.nodes[current_vertex], cur_tour.nodes[start_vertex2]);

        if added_sum - removed_sum > 5.0 {
            continue;
        }


        let test_fast = cur_tour.test_changes_fast(&added, &removed);
        if test_fast.is_none() {
            continue;
        }

        let mut actual_len = test_fast.unwrap();
        let start_len = cur_tour.get_len();
        println!("test {} {}", actual_len, start_len);

        if actual_len < start_len + 10.0 {
            let (_, cur_tour_path) = cur_tour.test_changes(&added, &removed).unwrap();
            cur_tour = cur_tour.make_new(cur_tour_path);
            cc += 1;
            println!("boo {}", cc);
            if cc == 30 {
                break;
            }
        }
    }
    let start_len = tour.get_len();
    let mut actual_len = cur_tour.get_len();
    let mut actual_real_len = cur_tour.get_real_len();

    println!("go2b {} {} ", start_len, actual_len);


    let mut last = 0;
    let mut fouls = 0;
    let mut added_v = vec!();
    let mut removed_v = vec!();
    let mut cand_buf = vec!();
    for i in 0..1_000_000_000 {
        if let Some((new_tour, _)) = do_opt(&mut cur_tour, candidates, pi, 0.0, base_limit, "heavy-", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
            if new_tour.get_path() != tour.get_path() {
                if new_tour.get_len() < actual_len {
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
        if i > 2_000_000 {
            break;
        }
        if actual_len < start_len {
            break;
        }
    }

    let mut last = 0usize;
    let mut cc = 0usize;
    let mut perm = (1..tour.get_path().len()-1).collect::<Vec<_>>();
    perm.shuffle(&mut rng);
    loop {
        if let Some(new_tour) = do_opt_all(&mut cur_tour, candidates, pi, base_limit, "heavyx-", &mut added_v, &mut removed_v, &mut cand_buf, perm[cc % perm.len()]) {
            cur_tour = new_tour;
            last = cc;
        }
        cc += 1;
        if cc - last > 50000 {
            break;
        }
        if cc % 10000 == 0 {
            println!("ccc {}", cc);
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
    #[structopt(short = "t", long = "temp", default_value = "0.00")]
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
    cand_file: String,

    #[structopt(short = "mpb", long = "max-penalty-bonus", default_value = "0.0")]
    max_penalty_bonus: f64,

    #[structopt(short = "ls", long = "length-slope", default_value = "0.0")]
    length_slope: f64,

    #[structopt(short = "lms", long = "length-min-slope", default_value = "0.0")]
    length_min_slope: f64,

    #[structopt(short = "bm", long = "bad-mods", default_value = "0.0")]
    bad_mods: f64,

    #[structopt(short = "pf", long = "pi-file", default_value = "../inputs/cities.pi.0")]
    pi_file: String,

    #[structopt(short = "seed", long = "seed", default_value = "4723")]
    base_seed: u64,

}

fn main() {
    println!("{} {} {}", our_rng().gen_range(0, 20), our_rng().gen_range(0, 20), our_rng().gen_range(0, 20));
    let opt = Config::from_args();
    let base_seed = opt.base_seed;

    let nodes = Arc::new(load_poses());

    let mut penalty_config: PenaltyConfig = Default::default();
    penalty_config.base_penalty = opt.penalty;
    penalty_config.max_penalty_bonus = opt.max_penalty_bonus;
    penalty_config.length_slope = opt.length_slope;
    penalty_config.length_min_slope = opt.length_min_slope;
    for i in 1..6 {
        penalty_config.bad_mods[i] = penalty_config.bad_mods[i-1] * opt.bad_mods;
        if i > 1 {
            penalty_config.bad_mods[10-i] = penalty_config.bad_mods[10-i+1] * opt.bad_mods;
        } else {
            penalty_config.bad_mods[9] = penalty_config.bad_mods[0] * opt.bad_mods;
        }
    }
    let bm_sum = penalty_config.bad_mods.iter().sum::<f64>();
    for i in 0..10 {
        penalty_config.bad_mods[i] /= bm_sum;
    }
    println!("bm {:?}", penalty_config.bad_mods);
    /*penalty_config.hash_mod = 223*10;
    penalty_config.hash_range = 223*9;*/

    let pi = load_pi2(nodes.len(), &opt.pi_file);
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
        let our_pi = pi.clone();
        let seed = base_seed + thread_id as u64;
        let handle = thread::spawn(move || {
            seed_rng(seed);
            /*thread::sleep(time::Duration::new(180, 0));*/
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some(new_tour_base) = do_opt2b(&mut our_tour, &our_candidates, &our_pi, &prefix, base_limit) {
                    //println!("new len {}", new_tour.get_len());
                    {
                        let main_tour = main_tour_mutex.lock().unwrap().clone();

                        let new_tour = merge(&new_tour_base, &main_tour, &prefix, main_tour.get_penalty_config());
                        if new_tour.get_len() < main_tour_mutex.lock().unwrap().get_len() {
                            println!("acceptxa {} real {} {}", new_tour.get_len(), new_tour.get_real_len(), Local::now().format("%Y-%m-%dT%H:%M:%S"));
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


    for thread_id in opt.n_heavy_threads..opt.n_threads + opt.n_heavy_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let base_limit = opt.base_limit;
        let our_pi = pi.clone();
        let seed = base_seed + thread_id as u64;
        let handle = thread::spawn(move || {
            seed_rng(seed);
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            let mut added_v = vec!();
            let mut removed_v = vec!();
            let mut cand_buf = vec!();
            loop {
                if let Some((new_tour, pr)) = do_opt(&mut our_tour, &our_candidates, &our_pi, temp, base_limit, "", &mut added_v, &mut removed_v, &mut cand_buf, &HashSet::new(), 0) {
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
                if cc % 100000 == 0 {
                    println!("cc {} {} {}", cc, thread_id, Local::now().format("%Y-%m-%dT%H:%M:%S"));
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

    for thread_id in opt.n_threads + opt.n_heavy_threads..opt.n_threads + opt.n_heavy_threads + opt.n_weak_threads {
        let main_tour_mutex = Arc::clone(&tour);
        let main_tour_hash = Arc::clone(&tour_hash);
        let our_candidates = candidates_w.clone();
        let prefix = format!("{}-tmp-{}", opt.save_to.clone(), thread_id);
        let base_limit = opt.base_limit;
        let our_pi = pi.clone();
        let seed = base_seed + thread_id as u64;
        let handle = thread::spawn(move || {
            seed_rng(seed);
            /*thread::sleep(time::Duration::new(90, 0));*/
            let mut cc = 0;
            let mut our_tour = main_tour_mutex.lock().unwrap().clone();
            let mut our_tour_hash = our_tour.hash();
            loop {
                if let Some(new_tour_base) = do_opt_break_local(&mut our_tour, &our_candidates, &our_pi, &prefix, base_limit, thread_id) {
                    //println!("new len {}", new_tour.get_len());
                    {
                        let main_tour = main_tour_mutex.lock().unwrap().clone();

                        let new_tour = merge(&new_tour_base, &main_tour, &prefix, main_tour.get_penalty_config());
                        if new_tour.get_len() <  main_tour_mutex.lock().unwrap().get_len() {
                            println!("acceptxw {} {} real {} {}", thread_id % n_local_configs, new_tour.get_len(), new_tour.get_real_len(), Local::now().format("%Y-%m-%dT%H:%M:%S"));
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
