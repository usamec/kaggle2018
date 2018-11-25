extern crate updater;
extern crate rand;

use updater::*;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use rand::prelude::*;
use std::collections::HashSet;


fn do_opt(tour: &mut Tour, candidates: &[Vec<usize>]) -> Option<Tour> {
    let mut rng = rand::thread_rng();
    let start_path_pos = rng.gen_range(1, tour.get_path().len() - 1);
    let start_vertex = tour.get_path()[start_path_pos];
    let start_vertex2 = tour.get_path()[start_path_pos+1];

    let mut removed = Vec::new();
    removed.push((start_vertex, start_vertex2));
    let mut added = Vec::new();

    let mut current_vertex = start_vertex;
    for _ in 0..2 {
        let mut next_vertex = 0;
        loop {
            next_vertex = *candidates[current_vertex].choose(&mut rng).unwrap();
            if next_vertex != 0 {
                break;
            }
        }
        added.push((current_vertex, next_vertex));

        loop {
            current_vertex = tour.rand_neighbour(next_vertex);
            if current_vertex != 0 {
                break;
            }
        }
        removed.push((next_vertex, current_vertex));
    }

    added.push((current_vertex, start_vertex2));

    //println!("{:?}", removed);
    //println!("{:?}", added);

    let test_fast = tour.test_changes_fast(&added, &removed);
    if let Some(len) = test_fast {
        if len < tour.get_len() {
            let test = tour.test_changes(&added, &removed);
            if let Some((res, p)) = test {
                //println!("{:?}", test);
                if res < tour.get_len() {
                    Some(tour.make_new(p, ))
                } else {
                    None
                }
            } else {
                None
            }
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
    let mut tour = Tour::new(load_tour("../outputs/candidate.csv"), nodes.clone(), primes.clone());
    let candidates = load_candidates();
    println!("Hello, world! {:?} {:?} {:?}", nodes.len(), tour.get_path().len(), candidates.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour.get_path(), &primes));
    println!("{:?}", tour.check_nodes_edges().unwrap().0);


    let cur_len = verify_and_calculate_len(&nodes, &tour.get_path(), &primes);
    let mut cc = 0;
    loop {
        if let Some(new_tour) = do_opt(&mut tour, &candidates) {
            println!("new len {}", new_tour.get_len());
            tour = new_tour;
            tour.save("../outputs/kopt4.csv");
        }
        cc += 1;
        if cc % 1000000 == 0 {
            println!("cc {}", cc);
        }
    }
}