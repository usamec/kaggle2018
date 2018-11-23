extern crate updater;

use updater::*;

fn main() {
    let nodes = load_poses();
    let tour = load_tour("../outputs/working.csv");
    let primes = get_primes(nodes.len());
    println!("Hello, world! {:?} {:?}", nodes.len(), tour.len());
    println!("{:?}", &primes[..20]);
    println!("total len {:?}", verify_and_calculate_len(&nodes, &tour, &primes));

    println!("total primes {} {}", primes.iter().map(|&x| x as usize).sum::<usize>(), (primes.iter().map(|&x| x as usize).sum::<usize>() as f64) / (nodes.len() as f64));

    let mut base_len = 0.0;
    let mut penalty_len = 0.0;
    let mut primes_at_10th = 0;
    let mut nonprimes_at_10th = 0;

    for i in 0..tour.len()-1 {
        let current_len = dist(nodes[tour[i]], nodes[tour[i+1]]);
        base_len += current_len;
        if (i + 1) % 10 == 0 {
            if !primes[tour[i]] {
                penalty_len += current_len * 0.1;
                nonprimes_at_10th += 1;
            } else {
                primes_at_10th += 1;
            }
        }
    }
    println!("base len {}", base_len);
    println!("penalty len {}", penalty_len);
    println!("primes at 10th {} {}", primes_at_10th, (primes_at_10th as f64) / ((primes_at_10th + nonprimes_at_10th) as f64));
    println!("avg step {}", base_len / nodes.len() as f64);
    println!("avg step at 10th {}", 10.0 * penalty_len / nonprimes_at_10th as f64);
}