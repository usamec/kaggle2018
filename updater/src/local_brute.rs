use dist;
use UnionFind;
use calculate_len;
use rand::prelude::*;
use super::*;

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
            current_len += get_penalty(current_len, 1 + offset, cur, primes);
            (Some((vec!(cur, target), current_len)), 1)
        } else {
            let mut best_len = -1f64;
            let mut best_path = Vec::new();
            let mut total_steps = 1;
            let mut rng = rand::thread_rng();
            left.shuffle(&mut rng);
            for &next in left.iter() {
                let mut current_len = dist(nodes[cur], nodes[next]);
                current_len += get_penalty(current_len, offset + 1, cur, primes);
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

                if total_steps > 2000000 {
                    println!("out! {}", total_steps);
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

pub fn full_optim(slice: &mut[usize], nodes: &[(f64,f64)], primes: &[bool], offset: usize) -> bool {
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