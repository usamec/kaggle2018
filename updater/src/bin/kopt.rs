extern crate updater;
extern crate rand;

use updater::*;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use rand::prelude::*;
use std::collections::HashSet;


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
    for _ in 0..*[2,3,4,5].choose(&mut rng).unwrap() {
        let mut next_vertex = 0;
        loop {
            next_vertex = candidates[current_vertex].choose_weighted(&mut rng, |x| x.1).unwrap().0;
            if next_vertex != 0 {
                break;
            }
        }
        added_sum += dist(tour.nodes[current_vertex], tour.nodes[next_vertex]);
        added.push((current_vertex, next_vertex));

        if added_sum - removed_sum > 10.0 {
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

    if added_sum - removed_sum > 10.0 {
        return None
    }

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
            Some(tour.make_new(p, ))
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    let nodes = Rc::new(load_poses());
    let primes = Rc::new(get_primes(nodes.len()));
    let mut tour = Tour::new(load_tour("../outputs/best.csv"), nodes.clone(), primes.clone());
    let candidates = load_candidates();
    let candidates_w = candidates.iter().enumerate().map(|(i, cc)| {
        cc.iter().map(|&c| (c, 1.0 / dist(nodes[i], nodes[c]).ln_1p())).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.get_path().len(), candidates.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour.get_path(), &primes));
    println!("{:?}", tour.check_nodes_edges().unwrap().0);


    let cur_len = verify_and_calculate_len(&nodes, &tour.get_path(), &primes);
    let mut cc = 0;
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
    }
}