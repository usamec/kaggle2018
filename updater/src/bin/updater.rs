extern crate rand;
extern crate updater;

use std::fs::File;
use std::io::prelude::*;
use rand::prelude::*;

use updater::*;

fn save(tour: &[usize]) {
    let mut output = File::create("../outputs/working.csv").unwrap();
    writeln!(output, "Path");
    tour.iter().for_each(|x| {
        writeln!(output, "{}", x);
    });
}

fn shift(slice: &mut [usize]) {
    let tmp = slice[slice.len()-1];
    for i in (0..slice.len()-1).rev() {
        slice[i+1] = slice[i];
    }
    slice[0] = tmp;
}

fn shiftl(slice: &mut [usize]) {
    let tmp = slice[0];
    for i in 0..slice.len()-1 {
        slice[i] = slice[i+1];
    }
    slice[slice.len()-1] = tmp;
}

fn get_lower_bound(cur: usize, left: &[usize], target: usize, nodes: &[(f64,f64)]) -> f64 {
    let mut all_nodes = Vec::new();
    all_nodes.push(nodes[cur]);
    all_nodes.extend(left.iter().map(|&x| nodes[x]));
    all_nodes.push(nodes[target]);

    let mut edges = Vec::new();
    for i in 0..all_nodes.len() {
        for j in 0..i {
            edges.push((dist(all_nodes[i], all_nodes[j]), i, j));
        }
    }
    edges.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let mut total = 0.0;
    let mut joins_left = all_nodes.len() - 1;
    let mut uf = UnionFind::new(all_nodes.len());
    for (l, a, b) in edges {
        if uf.join(a, b) {
            total += l;
            joins_left -= 1;
            if joins_left == 0 {
                break;
            }
        }
    }
    total
}

fn full_optim_inner(cur: usize, mut left: Vec<usize>, target: usize, offset: usize, nodes: &[(f64,f64)], primes: &[bool], upper_bound: f64) -> (Option<(Vec<usize>, f64)>, usize) {
    if upper_bound < get_lower_bound(cur, &left, target, nodes) {
        (None, 1)
    } else {
        if left.is_empty() {
            let mut current_len = dist(nodes[cur], nodes[target]);
            if (offset + 1) % 10 == 0 && !primes[cur] {
                current_len *= 1.1;
            }
            (Some((vec!(cur, target), current_len)), 1)
        } else {
            let mut best_len = -1f64;
            let mut best_path = Vec::new();
            let mut total_steps = 1;
            let mut rng = rand::thread_rng();
            left.shuffle(&mut rng);
            for &next in left.iter() {
                let mut current_len = dist(nodes[cur], nodes[next]);
                if (offset + 1) % 10 == 0 && !primes[cur] {
                    current_len *= 1.1;
                }
                let mut left2 = left.iter().filter(|&&x| x != next).map(|x| *x).collect::<Vec<_>>();
                let (maybe_res, steps) = full_optim_inner(next, left2, target, offset + 1, nodes, primes, upper_bound - current_len);
                total_steps += steps;
                if let Some((path, path_len)) = maybe_res {
                    let total_len = current_len + path_len;
                    if best_len < 0.0 || best_len > total_len {
                        best_len = total_len;
                        best_path = path;
                    }
                }

                if total_steps > 50000 {
                    break
                }
            }
            if best_len > 0.0 {
                best_path.push(cur);
                best_path.rotate_right(1);
                (Some((best_path, best_len)), total_steps)
            } else {
                (None, total_steps)
            }
        }
    }
}

fn full_optim(slice: &mut[usize], nodes: &[(f64,f64)], primes: &[bool], offset: usize) -> bool {
    //println!("full optim start {}", slice.len());

    let left = slice[1..slice.len()-1].to_owned();
    let start = slice[0];
    let end = *slice.last().unwrap();

    let best_len = calculate_len(&nodes, &slice, &primes, offset);

    let (maybe_res, total_steps) = full_optim_inner(start, left, end, offset, nodes, primes, best_len + 0.0001);
    if let Some((path, path_len)) = maybe_res {
        if path_len < best_len - 0.0001 {
            println!("len {} {} {} {} {}", path_len, best_len, slice.len(), offset % 10, total_steps);
            println!("{:?}", slice);
            println!("{:?}", path);
            for i in 0..slice.len() {
                slice[i] = path[i];
            }
            //println!("full optim end {} {}", slice.len(), total_steps);
            true
        } else {
            //println!("full optim end {} {}", slice.len(), total_steps);
            false
        }
    } else {
        //println!("full optim end {} {}", slice.len(), total_steps);
        false
    }
}

fn main() {
    let nodes = load_poses();
    let tour = load_tour("../outputs/working.csv");
    let primes = get_primes(nodes.len());
    println!("Hello, world! {:?} {:?}", nodes.len(), tour.len());
    println!("{:?}", &primes[..20]);
    println!("{:?}", verify_and_calculate_len(&nodes, &tour, &primes));

    let mut current_tour = tour.clone();
    let mut rng = rand::thread_rng();
    let mut cur_len = verify_and_calculate_len(&nodes, &current_tour, &primes);


    for _outer_iter in 0..1000usize {
        for iter in 0..100 {
            let size = rng.gen_range(15, 36);
            let start = rng.gen_range(1, current_tour.len() - size - 1);
            //let start = (iter) % (current_tour.len() - size - 2) + 1;
            let end = start + size;

            let mut slice_and_padding = current_tour[start - 1..end + 1].to_owned();
            let old_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
            if full_optim(&mut slice_and_padding, &nodes, &primes, start - 1) {
                let new_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
                if new_len < old_len/* || ((old_len - new_len) / temp).exp() > rng.gen::<f64>()*/ {
                    println!("boom {} {} {} {} {}", old_len, new_len, old_len - new_len, iter, size);
                    let mut new_tour = current_tour.clone();
                    {
                        let mut slice = &mut new_tour[start - 1..end + 1];
                        for i in 0..slice_and_padding.len() {
                            slice[i] = slice_and_padding[i];
                        }
                    }
                    let new_len = verify_and_calculate_len(&nodes, &new_tour, &primes);
                    println!("new_len {}", new_len);
                    if new_len < cur_len {
                        println!("better {:?}", new_len);
                    }
                    cur_len = new_len;
                    current_tour = new_tour;
                    println!("saving");
                    save(&current_tour);
                    println!("done");
                }
            }
            if iter % 10 == 0 {
                println!("iter {} {}", iter, cur_len);
            }
        }
        for iter in 0..1_000_00 {
            let temp = 0.1f64;
            let size = rng.gen_range(2, 500);
            let start = rng.gen_range(1, current_tour.len() - size - 1);
            let end = start + size;
            let se_dist = dist(nodes[start], nodes[end-1]);
            if se_dist < 1000.0 {
                let op = rng.gen_range(0, if size > 2 { 3 } else { 3 });

                let inner_op: Box<Fn(&mut [usize])> = match op {
                    0 => Box::new(|x: &mut [usize]| {
                        x.reverse();
                    }),
                    1 => Box::new(|x: &mut [usize]| {
                        shift(x);
                    }),
                    _ => Box::new(|x: &mut [usize]| {
                        shiftl(x);
                    })
                };

                let mut slice_and_padding = current_tour[start - 1..end + 1].to_owned();
                let old_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
                {
                    let mut slice = &mut slice_and_padding[1..size + 1];
                    inner_op(slice);
                }
                let new_len = calculate_len(&nodes, &slice_and_padding, &primes, start - 1);
                if new_len < old_len || ((old_len - new_len) / temp).exp() > rng.gen::<f64>() {
                    println!("boom {} {} {} {} {} {}", op, old_len, new_len, old_len - new_len, iter, size);
                    let mut new_tour = current_tour.clone();
                    {
                        let mut slice = &mut new_tour[start..end];
                        inner_op(slice);
                    }
                    let new_len = verify_and_calculate_len(&nodes, &new_tour, &primes);
                    println!("new_len {}", new_len);
                    if new_len < cur_len {
                        println!("better {:?}", new_len);
                    }
                    cur_len = new_len;
                    current_tour = new_tour;
                    println!("saving");
                    save(&current_tour);
                    println!("done");
                }
            }
            if iter % 1000000 == 0 {
                println!("iter {} {}", iter, cur_len);
            }
        }
    }
}
