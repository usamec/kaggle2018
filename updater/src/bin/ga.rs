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
                    let cycles = tour.count_cycles(&added, &removed);
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

// From http://mx.nthu.edu.tw/~ckting/pubs/evocop2004.pdf
// but assymetric version, sucks hard
fn erxta(t1: &BareTour, t2: &BareTour) -> Option<BareTour> {
    println!("erxt start {} {}", t1.get_len(), t2.get_len());
    
    let mut in1 = vec![0; t1.get_path().len() - 1];
    let mut out1 = vec![0; t1.get_path().len() - 1];
    for i in 0..t1.get_path().len() - 1 {
        in1[t1.get_path()[i+1]] = t1.get_path()[i];
        out1[t1.get_path()[i]] = t1.get_path()[i+1];
    }

    let mut in2 = vec![0; t2.get_path().len() - 1];
    let mut out2 = vec![0; t1.get_path().len() - 1];
    for i in 0..t2.get_path().len() - 1 {
        in2[t2.get_path()[i+1]] = t2.get_path()[i];
        out2[t2.get_path()[i]] = t2.get_path()[i+1];
    }

    let mut used = vec![false; t1.get_path().len() - 1];

    used[0] = true;

    let mut out_path = vec![0usize];

    let mut skips = 0;
    let mut used1 = 0;
    let mut used2 = 0;
    
    while out_path.len() < t1.get_path().len() -1 {
        let mut cur = *out_path.last().unwrap();
        let mut priority1 = used2;
        if used[out1[cur]] {
            priority1 -= 1_000_000_000;
        } else {
            let next = out1[cur];
            if in1[next] == in2[next] || (used[in1[next]] && used[in2[next]]) {
                priority1 += 5;
            }
        }

        let mut priority2 = used1;
        if used[out2[cur]] {
            priority2 -= 1_000_000_000;
        } else {
            let next = out2[cur];
            if in1[next] == in2[next] || (used[in1[next]] && used[in2[next]]) {
                priority2 += 5;
            }
        }

        if used[out1[cur]] && used[out2[cur]] {
            let mut next = out1[cur];
            while used[next] {
                next = out1[next];
            }

            if next == 0 {
                println!("total fail");
                return None
            } else {
                assert!(used[next] == false);
                out_path.push(next);
                used[next] = true;
                skips += 1;
            }
        } else {
            //println!("prio {} {}", priority1, priority2);
            if priority1 > priority2 {
                used1 += 1;
                let next = out1[cur];

                assert!(used[next] == false);
                out_path.push(next);
                used[next] = true;
            } else {
                used2 += 1;
                let next = out2[cur];
                assert!(used[next] == false);
                out_path.push(next);
                used[next] = true;
            }
        }
    }

    out_path.push(0);
    println!("ol {} t1l {}", out_path.len(), t1.get_path().len());

    let new_tour = t1.make_new(out_path);

    println!("new len {}", new_tour.get_len());

    Some(new_tour)
}

// From http://mx.nthu.edu.tw/~ckting/pubs/evocop2004.pdf
// but assymetric version, sucks hard
fn erxt(t1: &BareTour, t2: &BareTour) -> Option<BareTour> {
    println!("erxt start {} {}", t1.get_len(), t2.get_len());
    let mut in1 = vec![0; t1.get_path().len() - 1];
    let mut out1 = vec![0; t1.get_path().len() - 1];
    for i in 0..t1.get_path().len() - 1 {
        in1[t1.get_path()[i+1]] = t1.get_path()[i];
        out1[t1.get_path()[i]] = t1.get_path()[i+1];
    }

    let mut all_edges: Vec<Vec<usize>> = (0..(t1.get_path().len()-1)).map(|_| Vec::new()).collect();
    let mut edges: Vec<Vec<usize>> = (0..(t1.get_path().len()-1)).map(|_| Vec::new()).collect();
    for i in 0..t1.get_path().len() - 1 {
        all_edges[t1.get_path()[i+1]].push(t1.get_path()[i]);
        all_edges[t1.get_path()[i]].push(t1.get_path()[i+1]);
        edges[t1.get_path()[i+1]].push(t1.get_path()[i]);
        edges[t1.get_path()[i]].push(t1.get_path()[i+1]);
    }

    for i in 0..t2.get_path().len() - 1 {
        all_edges[t2.get_path()[i+1]].push(t2.get_path()[i]);
        all_edges[t2.get_path()[i]].push(t2.get_path()[i + 1]);

        if !edges[t2.get_path()[i+1]].contains(&t2.get_path()[i]) {
            edges[t2.get_path()[i+1]].push(t2.get_path()[i]);
        }
        if !edges[t2.get_path()[i]].contains(&t2.get_path()[i + 1]) {
            edges[t2.get_path()[i]].push(t2.get_path()[i + 1]);
        }
    }

    let mut used = vec![false; t1.get_path().len() - 1];

    used[0] = true;

    let mut out_path = vec![0usize];
    let mut skips = 0;
    let mut used1 = 0;
    let mut used2 = 0;

    while out_path.len() < t1.get_path().len() -1 {
        let mut cur = *out_path.last().unwrap();
        let mut next = 1_000_000;
        let mut best_prio = -1_000_000_000;
        let mut source = 5;

        for i in 0..4 {
            let maybe_next = all_edges[cur][i];
            if used[maybe_next] {
                continue
            }

            let mut is_common = 0;
            for j in 0..4 {
                if i != j && all_edges[cur][i] == all_edges[cur][j] {
                    is_common = 1;
                }
            }

            let not_used_choices: i32 = edges[maybe_next].iter().map(|&x| used[x] as i32).sum();
            let cur_prio = 2*is_common - not_used_choices + (if i < 2 { used2 } else { used1 });
            if cur_prio > best_prio {
                next = maybe_next;
                source = i;
                best_prio = cur_prio;
            }
        }
        if next == 1_000_000 {
            skips += 1;
            next = out1[cur];
            while used[next] {
                next = out1[next];
            }
        }

        assert!(used[next] == false);
        used[next] = true;
        out_path.push(next);
        if source == 5 {
            skips += 1;
        } else if source < 2 {
            used1 += 1;
        } else {
            used2 += 1;
        }
    }


    out_path.push(0);

    let new_tour = t1.make_new(out_path);

    println!("new len {} {} {} {}", new_tour.get_len(), used1, used2, skips);

    Some(new_tour)
}

fn eax(t1: &BareTour, t2: &BareTour) -> Option<BareTour> {
    println!("eax start {} {}", t1.get_len(), t2.get_len());

    let mut in1 = vec![0; t1.get_path().len() - 1];
    let mut out1 = vec![0; t1.get_path().len() - 1];
    for i in 0..t1.get_path().len() - 1 {
        in1[t1.get_path()[i+1]] = t1.get_path()[i];
        out1[t1.get_path()[i]] = t1.get_path()[i+1];
    }

    let mut in2 = vec![0; t2.get_path().len() - 1];
    let mut out2 = vec![0; t1.get_path().len() - 1];
    for i in 0..t2.get_path().len() - 1 {
        in2[t2.get_path()[i+1]] = t2.get_path()[i];
        out2[t2.get_path()[i]] = t2.get_path()[i+1];
    }

    let mut used1 = vec![false; t1.get_path().len() - 1];
    let mut used2 = vec![false; t1.get_path().len() - 1];

    let mut ab_cycles: Vec<Vec<usize>> = Vec::new();

    while let Some((start, _)) = used1.iter().enumerate().find(|&(i, &x)| !x ) {
        let mut cur = start;
        let mut cycle = Vec::new();
        loop {
            assert!(used1[cur] == false);
            used1[cur] = true;
            cycle.push(cur);
            cycle.push(out1[cur]);
            cur = out2[out1[cur]];
            if cur == start {
                break
            }
        }
        ab_cycles.push(cycle);
    }

    ab_cycles = ab_cycles.into_iter().filter(|x| !x.contains(&0)).collect();
    ab_cycles.sort_unstable_by_key(|a| -(a.len() as i64));
    println!("got {} abcycles", ab_cycles.len());
    println!("lens {:?}", ab_cycles.iter().map(|x| x.len()).collect::<Vec<_>>());
    println!("sum {:?} {}", ab_cycles.iter().map(|x| x.len()).sum::<usize>(), t1.get_path().len() * 2);

    let tt1 = t1.to_tour();

    let cycle = &ab_cycles[3];
    println!("cycle len {}", cycle.len());
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for i in 0..cycle.len() - 1 {
        if i % 2 == 0 {
            removed.push((cycle[i], cycle[i+1]));
        } else {
            added.push((cycle[i], cycle[i+1]));
        }
    }
    added.push((*cycle.last().unwrap(), cycle[0]));

    let clean_added = added.iter().filter(|&&(a, b)| !removed.contains(&(a,b))).map(|x| *x).collect::<Vec<_>>();
    let clean_removed = removed.iter().filter(|&&(a, b)| !added.contains(&(a,b))).map(|x| *x).collect::<Vec<_>>();
    println!("{} {} {} {}", added.len(), removed.len(), clean_added.len(), clean_removed.len());scre
    println!("made {} cycles", tt1.count_cycles(&clean_added, &clean_removed));

    None
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
    let mut rng = rand::thread_rng();
    let opt = Config::from_args();

    let nodes = Arc::new(load_poses());

    unsafe {
        penalty_config.base_penalty = opt.penalty;
    }


    let primes = Arc::new(get_primes(nodes.len()));
    let base_tours = fs::read_dir(&opt.load_from).unwrap().map(|x| BareTour::new(load_tour(&x.unwrap().path().to_str().unwrap()), nodes.clone(), primes.clone())).collect::<Vec<_>>();
    println!("loaded {} base_tours", base_tours.len());

    //let base_tour = erxta(&base_tours.choose(&mut rng).unwrap(), &base_tours.choose(&mut rng).unwrap()).unwrap().to_tour();
    let base_tour = eax(&base_tours.choose(&mut rng).unwrap(), &base_tours.choose(&mut rng).unwrap()).unwrap().to_tour();
    return;


    //let base_tour = Tour::new(load_tour(&opt.load_from), nodes.clone(), primes.clone());
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

    let best_tour_mutex = Arc::new(Mutex::new(base_tour.clone()));
    let best_tour_hash =  Arc::new(AtomicUsize::new(best_tour_mutex.lock().unwrap().hash()));
    let temp = 0.003;

    {
        let our_base_tour = base_tour.clone();
        let our_candidates = candidates_w.clone();
        let prefix = opt.save_to.clone();
        let base_limit = opt.base_limit;
        /*let handle = thread::spawn(move ||*/ {
            for cand in 0..1 {
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
                        println!("cc {} {} {} / {}", cc, 1, our_tour.get_len(), our_tour.get_real_len());
                    }
                    if cc > last_best + 1_000_000{
                        break;
                    }
                }
                println!("fin {}", our_tour.get_len());
                /*println!("saving tour {} with len {} / {}",  thread_id*cands_per_thread+cand, our_tour.get_len(), our_tour.get_real_len());
                our_tour.save(&format!("{}/ga-{}.csv", prefix, thread_id*cands_per_thread+cand));*/
            }

        };
    }

    /*let mut handles = vec!();

    // writer thread
    {
        let main_tour_mutex = Arc::clone(&best_tour_mutex);
        let main_tour_hash = Arc::clone(&best_tour_hash);
        let prefix = opt.save_to.clone();
        let handle = thread::spawn(move || {
            let mut our_tour_hash = best_tour_hash.load(Ordering::Relaxed);
            let mut best_len = best_tour_mutex.lock().unwrap().get_len();
            let mut best_real_len = best_tour_mutex.lock().unwrap().get_real_len();

            loop {
                let cur_hash = best_tour_hash.load(Ordering::Relaxed);
                if cur_hash != our_tour_hash {
                    println!("saving");
                    let main_tour = best_tour_mutex.lock().unwrap().clone();
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
