pub struct UnionFind {
    boss: Vec<usize>
}

impl UnionFind {
    pub fn new(n: usize) -> UnionFind {
        let mut boss = vec![0; n];
        for i in 0..n {
            boss[i] = i;
        }
        UnionFind { boss }
    }

    pub fn get(&mut self, x: usize) -> usize {
        if self.boss[x] == x {
            x
        } else {
            let local_boss = self.boss[x];
            let boss = self.get(local_boss);
            self.boss[x] = boss;
            boss
        }
    }

    pub fn join(&mut self, a: usize, b: usize) -> bool {
        let ba = self.get(a);
        let bb = self.get(b);
        if ba == bb {
            false
        } else {
            self.boss[ba] = bb;
            true
        }
    }
}