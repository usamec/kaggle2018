use std::io::BufWriter;
use std::sync::Arc;
use rand::prelude::*;
use dist;
use verify_and_calculate_len;
use std::fs::File;
use std::io::prelude::*;
use std::iter;
use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

const NOEDGE: usize = 123456789;

#[derive(Clone)]
struct TwoEdges {
    edges: [usize;2]
}

impl TwoEdges {
    fn new() -> TwoEdges {
        TwoEdges { edges: [NOEDGE, NOEDGE] }
    }

    fn add(&mut self, node: usize) {
        if self.edges[0] == NOEDGE {
            self.edges[0] = node;
        } else if self.edges[1] == NOEDGE {
            self.edges[1] = node;
        } else {
            panic!("too many edges");
        }
    }

    fn get(&self, ind: usize) -> Option<usize> {
        if self.edges[ind] == NOEDGE {
            None
        } else {
            Some(self.edges[ind])
        }
    }

    fn has(&self, node: usize) -> bool {
        self.edges[0] == node || self.edges[1] == node
    }

    fn remove(&mut self, node: usize) {
        if self.edges[0] == node {
            self.edges[0] = NOEDGE;
        } else if self.edges[1] == node {
            self.edges[1] = NOEDGE;
        } else {
            panic!("edge not found");
        }
    }

    fn rand(&self) -> usize {
        let mut rng = rand::thread_rng();
        self.edges[rng.gen_range(0, 2)]
    }
}

#[derive(Clone)]
pub struct BareTour {
    pub nodes: Arc<Vec<(f64,f64)>>,
    primes: Arc<Vec<bool>>,
    path: Vec<usize>,
    cur_len: f64,
    cur_real_len: f64
}

impl BareTour {
    pub fn new(path: Vec<usize>, nodes: Arc<Vec<(f64, f64)>>, primes: Arc<Vec<bool>>) -> BareTour {
        let (cur_len, cur_real_len) = verify_and_calculate_len(&nodes, &path, &primes);

        BareTour { nodes, path, primes, cur_len, cur_real_len }
    }

    pub fn get_path(&self) -> &[usize] {
        &self.path
    }

    pub fn get_len(&self) -> f64 {
        self.cur_len
    }

    pub fn get_real_len(&self) -> f64 {
        self.cur_real_len
    }

    pub fn make_new(&self, path: Vec<usize>) -> BareTour {
        BareTour::new(path, self.nodes.clone(), self.primes.clone())
    }

    pub fn to_tour(&self) -> Tour {
        Tour::new(self.path.clone(), self.nodes.clone(), self.primes.clone())
    }
}

#[derive(Clone)]
pub struct Tour {
    pub nodes: Arc<Vec<(f64,f64)>>,
    primes: Arc<Vec<bool>>,
    path: Vec<usize>,
    inv: Vec<usize>,
    per_nodes_edges: Vec<TwoEdges>,
    cur_len: f64,
    cur_real_len: f64,
    prefix_lens: Vec<f64>,
    prefix_lens_offset: [Vec<f64>; 10],
    prefix_lens_offset_rev: [Vec<f64>; 10]
}

fn path_to_edges(path: &[usize]) -> Vec<TwoEdges> {
    let mut ret = (0..path.len()-1).map(|_| TwoEdges::new()).collect::<Vec<_>>();

    for (&first, &second) in path.iter().zip(path[1..].iter()) {
        ret[first].add(second);
        ret[second].add(first);
    }

    ret
}

impl Tour {
    pub fn new(path: Vec<usize>, nodes: Arc<Vec<(f64, f64)>>, primes: Arc<Vec<bool>>) -> Tour {
        let per_nodes_edges = path_to_edges(&path);
        let (cur_len, cur_real_len) = verify_and_calculate_len(&nodes, &path, &primes);
        let mut inv = vec![0; nodes.len()];
        for (i, &v) in path.iter().enumerate() {
            inv[v] = i;
        }

        let mut prefix_lens = vec![0.0];
        for (&a, &b) in path.iter().zip(path[1..].iter()) {
            let ll = prefix_lens.last().unwrap() + dist(nodes[a], nodes[b]);
            prefix_lens.push(ll);
        }

        let mut prefix_lens_offset = [vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0]];
        let mut prefix_lens_offset_rev = [vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0], vec![0.0]];
        for i in 0..path.len()-1 {
            let current_len = dist(nodes[path[i]], nodes[path[i+1]]);
            for j in 0..10 {
                let ll = prefix_lens_offset[j].last().unwrap() + get_penalty(current_len, i + 1 + j, path[i], &primes);
                prefix_lens_offset[j].push(ll);

                let ll = prefix_lens_offset_rev[j].last().unwrap() + get_penalty(current_len, i + 1 + j, path[i+1], &primes);
                prefix_lens_offset_rev[j].push(ll);
            }
        }

        Tour { nodes, path, primes, inv, per_nodes_edges, cur_len, prefix_lens, prefix_lens_offset, prefix_lens_offset_rev, cur_real_len }
    }

    pub fn penalties(&self) -> Vec<(usize, usize, f64)> {
        let mut out = Vec::new();
        for i in 0..self.path.len()-1 {
            let current_len = dist(self.nodes[self.path[i]], self.nodes[self.path[i+1]]);
            if (i + 1) % 10 == 0 {
                if !self.primes[self.path[i]] {
                    out.push((self.path[i], self.path[i+1],current_len * 0.1));
                } else {
                    out.push((self.path[i], self.path[i+1], 0.0));
                }
            }
        }
        out
    }

    pub fn hash(&self) -> usize {
        let mut hash = DefaultHasher::new();
        self.path.hash(&mut hash);
        hash.finish() as usize
    }

    pub fn make_new(&self, path: Vec<usize>) -> Tour {
        Tour::new(path, self.nodes.clone(), self.primes.clone())
    }

    pub fn recompute(self) -> Tour {
        Tour::new(self.path, self.nodes.clone(), self.primes.clone())
    }

    pub fn get_path(&self) -> &[usize] {
        &self.path
    }

    pub fn get_inv(&self) -> &[usize] { &self.inv }

    pub fn get_len(&self) -> f64 {
        self.cur_len
    }

    pub fn get_real_len(&self) -> f64 {
        self.cur_real_len
    }

    pub fn rand_neighbour(&self, node: usize) -> usize {
        self.per_nodes_edges[node].rand()
    }

    pub fn neighbours(&self, node: usize) -> [usize; 2] {
        self.per_nodes_edges[node].edges
    }

    pub fn largest_dist_to_neigh(&self, node: usize) -> f64 {
        self.per_nodes_edges[node].edges.iter().map(|&x| dist(self.nodes[node], self.nodes[x])).fold(0.0, f64::max)
    }

    fn apply_changes(&mut self, added: &[(usize, usize)], removed: &[(usize, usize)]) {
        for &(a, b) in removed {
            self.per_nodes_edges[a].remove(b);
            self.per_nodes_edges[b].remove(a);
        }

        for &(a, b) in added {
            self.per_nodes_edges[a].add(b);
            self.per_nodes_edges[b].add(a);
        }
    }

    pub fn check_nodes_edges(&self) -> Option<(f64, Vec<usize>)> {
        let mut cur = self.per_nodes_edges[0].get(0).unwrap();
        let mut prev = 0;
        let mut steps = 1;
        let mut total_len = dist(self.nodes[cur], self.nodes[prev]);
        let mut path = Vec::new();
        path.push(prev);
        path.push(cur);

        while cur != 0 {
            if self.per_nodes_edges[cur].get(0).unwrap() != prev {
                prev = cur;
                cur = self.per_nodes_edges[cur].get(0).unwrap();
            } else {
                prev = cur;
                cur = self.per_nodes_edges[cur].get(1).unwrap();
            }
            let current_len = dist(self.nodes[cur], self.nodes[prev]);
            total_len += current_len;
            total_len += get_penalty(current_len, 1 + steps, prev, &self.primes);
            steps += 1;
            path.push(cur);
        }

        if steps == self.nodes.len() {
            Some((total_len, path))
        } else {
            None
        }
    }

    pub fn test_changes(&mut self, added: &[(usize, usize)], removed: &[(usize, usize)]) -> Option<(f64, Vec<usize>)> {
        let mut duplicate = false;
        for i in 0..removed.len() {
            for j in 0..i {
                if removed[i] == removed[j] {
                    duplicate = true;
                }
                if removed[i].0 == removed[j].1 && removed[i].1 == removed[j].0 {
                    duplicate = true;
                }
            }
        }

        for i in 0..added.len() {
            for j in 0..removed.len() {
                if added[i] == removed[j] {
                    duplicate = true;
                }
                if added[i].0 == removed[j].1 && added[i].1 == removed[j].0 {
                    duplicate = true;
                }
            }
        }

        if !duplicate {
            /*let diff = added.iter().map(|&(a, b)| dist(self.nodes[a], self.nodes[b])).sum::<f64>()
                - removed.iter().map(|&(a, b)| dist(self.nodes[a], self.nodes[b])).sum::<f64>();
            if diff < 5000.0*/ {
                self.apply_changes(added, removed);

                let ret = self.check_nodes_edges();
                //println!("ret {:?}", ret);
                if let Some((res, p)) = ret {
/*                    if diff + self.cur_len - res > 3.0 {
                        println!("diff {} {} {}", diff, res, diff + self.cur_len - res);
                    }*/
                    self.apply_changes(removed, added);
                    Some((res, p))
                } else {
                    self.apply_changes(removed, added);
                    None
                }
            }/* else {
                None
            }*/
        } else {
            None
        }
    }

    pub fn get_dist_offset(&self, start: usize, end: usize, offset: usize) -> f64 {
        if end > start {
            let our_offset = (((offset as i32 - start as i32 % 10) + 10) % 10) as usize;
            self.prefix_lens[end] - self.prefix_lens[start] + self.prefix_lens_offset[our_offset][end] - self.prefix_lens_offset[our_offset][start]
        } else {
            let our_offset = (((-(start as i32 % 10) - offset as i32 + 9) + 20) % 10) as usize;
            -(self.prefix_lens[end] - self.prefix_lens[start] + self.prefix_lens_offset_rev[our_offset][end] - self.prefix_lens_offset_rev[our_offset][start])
        }
    }

    fn check_duplicates(added: &[(usize, usize)], removed: &[(usize, usize)]) -> bool {
        let mut duplicate = false;
        for i in 0..removed.len() {
            if removed[i].0 == 0 || removed[i].1 == 0 {
                return false;
            }
            for j in 0..i {
                if removed[i] == removed[j] {
                    duplicate = true;
                }
                if removed[i].0 == removed[j].1 && removed[i].1 == removed[j].0 {
                    duplicate = true;
                }
            }
        }

        for i in 0..added.len() {
            if added[i].0 == 0 || added[i].1 == 0 {
                return false;
            }
            for j in 0..removed.len() {
                if added[i] == removed[j] {
                    duplicate = true;
                }
                if added[i].0 == removed[j].1 && added[i].1 == removed[j].0 {
                    duplicate = true;
                }
            }
        }
        duplicate
    }

    pub fn count_cycles(&self, added: &[(usize, usize)], removed: &[(usize, usize)]) -> (usize, Vec<Vec<(usize, usize)>>) {
        if !Tour::check_duplicates(added, removed) {
            let mut removed_inds = removed.iter().map(|x| iter::once(self.inv[x.0]).chain(iter::once(self.inv[x.1]))).flatten().collect::<Vec<_>>();
            let added_inds = added.iter().map(|x| (self.inv[x.0], self.inv[x.1])).collect::<Vec<_>>();
            removed_inds.sort_unstable();

            /*println!("rem {:?}", removed);
            println!("add {:?}", added);
            println!("rid {:?}", removed_inds);
            println!("aid {:?}", added_inds);*/


            let mut used_added = added_inds.iter().map(|_| false).collect::<Vec<_>>();

            let mut current = removed_inds[0];
            loop {
                //println!("current {}", current);
                let added_pos = added_inds.iter().enumerate().find_map(|(i, x)| if (x.0 == current || x.1 == current) && used_added[i] == false { Some(i) } else { None }).unwrap();
                used_added[added_pos] = true;
                let next = if added_inds[added_pos].0 == current {
                    added_inds[added_pos].1
                } else {
                    added_inds[added_pos].0
                };

                let next_pos = removed_inds.iter().enumerate().find_map(|(i, &x)| if x == next { Some(i) } else { None }).unwrap();
                if next_pos == removed_inds.len() - 1 {
                    break;
                }
                if next_pos == 0 {
                    println!("rem {:?}", removed);
                    println!("add {:?}", added);
                    println!("rid {:?}", removed_inds);
                    println!("aid {:?}", added_inds);
                }
                assert!(next_pos != 0);

                if next_pos % 2 == 1 {
                    current = removed_inds[next_pos + 1];
                } else {
                    current = removed_inds[next_pos - 1];
                }
            }
            let mut cycles = 1;
            let mut all_cycle_parts = vec!();
            loop {
                let maybe_cycle_start = used_added.iter().enumerate().find_map(|(i, x)| if !x { Some(i) } else { None });
                let mut cycle_parts = vec!();

                match maybe_cycle_start {
                    None => break,
                    Some(cycle_start) => {
                        let start = added_inds[cycle_start].0;
                        //println!("start next cycle {}", start);
                        let mut current = start;
                        loop {
                            //println!("current {}", current);
                            let added_pos = added_inds.iter().enumerate().find_map(|(i, x)| if (x.0 == current || x.1 == current) && used_added[i] == false { Some(i) } else { None }).unwrap();
                            used_added[added_pos] = true;
                            let next = if added_inds[added_pos].0 == current {
                                added_inds[added_pos].1
                            } else {
                                added_inds[added_pos].0
                            };
                            //println!("next {}", next);

                            let next_pos = removed_inds.iter().enumerate().find_map(|(i, &x)| if x == next { Some(i) } else { None }).unwrap();

                            if next_pos == 0 {
                                println!("rem {:?}", removed);
                                println!("add {:?}", added);
                                println!("rid {:?}", removed_inds);
                                println!("aid {:?}", added_inds);
                            }
                            assert!(next_pos != 0);

                            if next_pos % 2 == 1 {
                                current = removed_inds[next_pos + 1];
                            } else {
                                current = removed_inds[next_pos - 1];
                            }

                            cycle_parts.push((next, current));

                            if current == start {
                                break;
                            }
                        }
                        cycles += 1;
                    }
                }
                all_cycle_parts.push(cycle_parts);
            }
            (cycles, all_cycle_parts)
            /*if used_added.iter().all(|&x| x) {
                1
            } else {
                47
            }*/
        } else {
            (1_000_000_000, vec!())
        }
    }

    pub fn test_changes_fast(&self, added: &[(usize, usize)], removed: &[(usize, usize)]) -> Option<f64> {
        if !Tour::check_duplicates(added, removed) {
            let mut removed_inds = removed.iter().map(|x| iter::once(self.inv[x.0]).chain(iter::once(self.inv[x.1]))).flatten().collect::<Vec<_>>();
            let added_inds = added.iter().map(|x| (self.inv[x.0], self.inv[x.1])).collect::<Vec<_>>();
            removed_inds.sort_unstable();
            /*println!("rem {:?}", removed);
            println!("add {:?}", added);
            println!("rid {:?}", removed_inds);
            println!("aid {:?}", added_inds);*/


            let mut used_added = added_inds.iter().map(|_| false).collect::<Vec<_>>();

            let mut total_len = 0.0f64;
            let mut current = removed_inds[0];
            total_len += self.get_dist_offset(0, removed_inds[0], 0);
            let mut cur_offset = removed_inds[0];
            cur_offset %= 10;
            loop {
                //println!("current {}", current);
                let added_pos = added_inds.iter().enumerate().find_map(|(i, x)| if (x.0 == current || x.1 == current) && used_added[i] == false { Some(i) } else { None }).unwrap();
                used_added[added_pos] = true;
                let next = if added_inds[added_pos].0 == current {
                    added_inds[added_pos].1
                } else {
                    added_inds[added_pos].0
                };

                let mut current_len = dist(self.nodes[added[added_pos].0], self.nodes[added[added_pos].1]);
                current_len += get_penalty(current_len, 1 + cur_offset, self.path[current], &self.primes);
                total_len += current_len;
                cur_offset += 1;
                cur_offset %= 10;

                let next_pos = removed_inds.iter().enumerate().find_map(|(i, &x)| if x == next { Some(i) } else { None }).unwrap();
                if next_pos == removed_inds.len() - 1 {
                    break;
                }
                if next_pos == 0 {
                    println!("rem {:?}", removed);
                    println!("add {:?}", added);
                    println!("rid {:?}", removed_inds);
                    println!("aid {:?}", added_inds);
                }
                assert!(next_pos != 0);
                if next_pos % 2 == 1 {
                    current = removed_inds[next_pos + 1];
                } else {
                    current = removed_inds[next_pos - 1];
                }

                total_len += self.get_dist_offset(next, current, cur_offset);
                cur_offset += (next as i32 - current as i32).abs() as usize;
                cur_offset %= 10;
            }
            total_len += self.get_dist_offset(*removed_inds.last().unwrap(), self.path.len()-1, cur_offset);
            //println!("{:?}", used_added);
            if used_added.iter().all(|&x| x) {
                Some(total_len)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn save(&self, filename: &str) {
        let mut output = BufWriter::new(File::create(filename).unwrap());
        writeln!(output, "Path");
        self.path.iter().for_each(|x| {
            writeln!(output, "{}", x);
        });
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use Tour;
    use get_primes;
    use std::sync::Arc;
    use std::iter::FromIterator;
    use calculate_len;

    #[test]
    fn test_tour_check() {
        let mut rng = rand::thread_rng();
        let nodes = (0..100).map(|_| (rng.gen_range(-1.0 ,1.0), rng.gen_range(-100.0 ,100.0))).collect::<Vec<_>>();

        let mut tour_path = Vec::from_iter(0..100);
        tour_path[1..].shuffle(&mut rng);
        tour_path.push(0);

        let primes = get_primes(100);

        let mut tour = Tour::new(tour_path, Arc::new(nodes), Arc::new(primes));

        let candidates = (0..100usize).map(|i| {
            (1..100usize).filter(|&j| j != i).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        for iter in 0..100000 {
            if iter % 1000 == 999 {
                println!("iter {}", iter);
            }
            let start_path_pos = rng.gen_range(1, tour.get_path().len() - 2);
            let start_vertex = tour.get_path()[start_path_pos];
            let start_vertex2 = tour.get_path()[start_path_pos+1];

            let mut removed = Vec::new();
            removed.push((start_vertex, start_vertex2));
            let mut added = Vec::new();

            let mut current_vertex = start_vertex;
            for _ in 0..5 {
                let next_vertex = *candidates[current_vertex].choose(&mut rng).unwrap();
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
            added.sort();
            removed.sort();

            let slow = tour.test_changes(&added, &removed);
            let fast = tour.test_changes_fast(&added, &removed);
            assert_eq!(slow.is_none(), fast.is_none());
            if let Some((slow_len, _p)) = slow {
                if let Some(fast_len) = fast {

                    //println!("s {} f {}", slow_len, fast_len);
                    assert_approx_eq!(slow_len, fast_len);
                }
            }

        }
    }

    #[test]
    fn test_dist() {
        let mut rng = rand::thread_rng();
        let nodes = Arc::new((0..100).map(|_| (rng.gen_range(-1.0 ,1.0), rng.gen_range(-100.0 ,100.0))).collect::<Vec<_>>());

        let mut tour_path = Vec::from_iter(0..100);
        tour_path[1..].shuffle(&mut rng);
        tour_path.push(0);

        let primes = Arc::new(get_primes(100));

        let tour = Tour::new(tour_path, nodes.clone(), primes.clone());

        for end in 0..100 {
            for start in 0..end {
                for offset in 0..10 {
                    let slow = calculate_len(&nodes, &tour.get_path()[start..end+1], &primes, offset);
                    let fast = tour.get_dist_offset(start, end, offset);
                    assert_approx_eq!(slow, fast);
                }
            }
        }
    }

    #[test]
    fn test_dist_rev() {
        let mut rng = rand::thread_rng();
        let nodes = Arc::new((0..100).map(|_| (rng.gen_range(-1.0 ,1.0), rng.gen_range(-100.0 ,100.0))).collect::<Vec<_>>());

        let mut tour_path = Vec::from_iter(0..100);
        tour_path[1..].shuffle(&mut rng);
        tour_path.push(0);
        println!("{:?}", tour_path);

        let primes = Arc::new(get_primes(100));

        let tour = Tour::new(tour_path, nodes.clone(), primes.clone());

        for end in 0..100 {
            for start in 0..end {
                for offset in 0..10 {
                    let slow = calculate_len(&nodes, &tour.get_path()[start..end+1].iter().map(|x| *x).rev().collect::<Vec<_>>(), &primes, offset);
                    let fast = tour.get_dist_offset(end, start, offset);
                    assert_approx_eq!(slow, fast);
                }
            }
        }
    }
}

