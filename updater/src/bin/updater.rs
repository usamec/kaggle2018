extern crate rand;
extern crate updater;

use std::fs::File;
use std::io::prelude::*;
use rand::prelude::*;

use updater::*;

fn save(tour: &[usize]) {
    let mut output = File::create("../outputs/test.csv").unwrap();
    writeln!(output, "Path");
    tour.iter().for_each(|x| {
        writeln!(output, "{}", x);
    });
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
            if new_len < cur_len {
                println!("better {:?}", new_len);
                Some((new_tour, new_len))
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
    let nodes = load_poses();
    let tour = load_tour("../outputs/candidate.csv");
    let primes = get_primes(nodes.len());
    println!("Hello, world! {:?} {:?}", nodes.len(), tour.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour, &primes));

    let mut current_tour = tour.clone();
    let mut rng = rand::thread_rng();
    let mut cur_len = verify_and_calculate_len(&nodes, &current_tour, &primes);


    for _outer_iter in 0..1000000000000usize {
        for iter in 0..10 {
            let size = rng.gen_range(35, 36);
            let start = rng.gen_range(1, current_tour.len() - size - 1);
            let maybe_new_tour = local_update(size, start, 0.0, &nodes, &current_tour, cur_len, &primes, full_optim);
            if let Some((new_tour, new_len)) = maybe_new_tour {
                cur_len = new_len;
                current_tour = new_tour;
                println!("saving");
                save(&current_tour);
                println!("done");
            }
            if iter % 10 == 0 {
                println!("iter {} {}", iter, cur_len);
            }
        }

        for size in 2..10000 {
            for start in 1..current_tour.len() - size - 1 {
                let temp = 0.001f64;
                //let size = rng.gen_range(2, 5000);
                //let start = rng.gen_range(1, current_tour.len() - size - 1);
                let end = start + size;
                let a_dist = dist(nodes[tour[start]], nodes[tour[start - 1]]);
                let b_dist = dist(nodes[tour[start]], nodes[tour[end - 1]]) + dist(nodes[tour[start - 1]], nodes[tour[end - 1]]);
                if b_dist < 6.0 * a_dist {
                    let op = rng.gen_range(0, if size > 2 { 3 } else { 3 });

                    let maybe_new_tour = match op {
                        0 => local_update(size, start, temp, &nodes, &current_tour, cur_len, &primes, |x, _, _, _| {
                            let len = x.len();
                            x[1..len-1].reverse();
                            true
                        }),
                        1 => local_update(size, start, temp, &nodes, &current_tour, cur_len, &primes, |x, _, _, _| {
                            let len = x.len();
                            x[1..len-1].rotate_right(1);
                            true
                        }),
                        _ => local_update(size, start, temp, &nodes, &current_tour, cur_len, &primes, |x, _, _, _| {
                            let len = x.len();
                            x[1..len-1].rotate_left(1);
                            true
                        })
                    };

                    if let Some((new_tour, new_len)) = maybe_new_tour {
                        cur_len = new_len;
                        current_tour = new_tour;
                        println!("saving");
                        save(&current_tour);
                        println!("done");
                    }
                }
            }
            if size % 1000 == 0 {
                println!("iter {} {}", size, cur_len);
            }

        }
    }
}
