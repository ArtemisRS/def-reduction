use randomize::{RandRangeU32, PCG32};

#[derive(Debug, Clone, Copy)]
struct Boss {
    hp: u16,
    def_lvl: u16,
    def_stat: u16,
    min_def_lvl: u16,
    mdr: u32,
}

impl Boss {
    fn new(hp: u16, def_lvl: u16, def_stat: u16, min_def_lvl: u16) -> Boss {
        Boss {
            hp,
            def_lvl,
            def_stat,
            min_def_lvl,
            mdr: Self::calc_mdr(def_lvl, def_stat),
        }
    }

    fn calc_mdr(def_lvl: u16, def_stat: u16) -> u32 {
        const RL: u32 = 400;
        let base_mdr = (def_lvl + 9) as u32 * (def_stat + 64) as u32;
        base_mdr * (RL * 4 + 1000) / 1000
    }

    fn reduce_def(&mut self, reduction: u16) {
        let max_reduction = self.def_lvl - self.min_def_lvl;
        if reduction > max_reduction {
            self.def_lvl = self.min_def_lvl;
        } else {
            self.def_lvl -= reduction;
        }
        self.mdr = Self::calc_mdr(self.def_lvl, self.def_stat);

        self.hit(reduction);
    }

    fn hit(&mut self, damage: u16) {
        if damage > self.hp {
            self.hp = 0;
        } else {
            self.hp -= damage;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Player {
    mar: u32,
    max_hit: u16,
    delay: u16,
}

impl Player {
    fn new(mar: u32, max_hit: u16, delay: u16) -> Player {
        Player {
            mar,
            max_hit,
            delay,
        }
    }
}

fn attack_until_dead(player: Player, mut boss: Boss, rng_gen: &mut PCG32) -> u16 {
    let mut ticks = 0;

    let mar_gen = RandRangeU32::new(0, player.mar);
    let mdr_gen = RandRangeU32::new(0, boss.mdr);
    let damage_gen = RandRangeU32::new(0, player.max_hit as u32);

    while boss.hp > 0 {
        ticks += player.delay;
        let mar = mar_gen.sample(rng_gen);
        let mdr = mdr_gen.sample(rng_gen);
        if mar > mdr {
            let damage = damage_gen.sample(rng_gen) as u16;
            boss.hit(damage);
        }
    }
    ticks
}

fn spec_bgs(player: Player, boss: &mut Boss, bgs_specs: u16, rng_gen: &mut PCG32) -> u16 {
    let mut specs = 0;
    for _ in 0..bgs_specs {
        let mar = RandRangeU32::new(0, player.mar).sample(rng_gen);
        let mdr = RandRangeU32::new(0, boss.mdr).sample(rng_gen);
        if mar > mdr {
            let damage = RandRangeU32::new(0, player.max_hit as u32).sample(rng_gen) as u16;
            boss.reduce_def(damage);
            specs += damage;
        }
    }
    specs
}

fn simulate_n(trials: usize, bgs: u16, rng_gen: &mut PCG32) {
    let mut times = Vec::with_capacity(trials);
    let mut misses = Vec::new();

    //let bgs_spec = Player::new(36814 * 2, 77, 6); //torva, fero, torture
    let bgs_spec = Player::new(36814 * 2, 75, 6); //bandos, fero, torture
    //let tbow = Player::new(53032, 80, 5); //masori, vambs, ame, no helm/boots
    let tbow = Player::new(49136, 76, 5); //arma, vambs, ame, prims, neit
    let boss_slash = Boss::new(571, 180, 40, 120);

    for _ in 0..trials {
        let mut temp_boss = boss_slash.clone();
        let drain = spec_bgs(bgs_spec, &mut temp_boss, bgs, rng_gen);
        let boss_range = Boss::new(temp_boss.hp, temp_boss.def_lvl, 20, 120);
        let ttk = attack_until_dead(tbow.clone(), boss_range, rng_gen);
        times.push(ttk + 6 * bgs);
        if drain == 0 {
            misses.push(ttk + 6 * bgs);
        }
    }
    let sum = times.iter().fold(0u64, |sum, i| sum + (*i as u64));
    let avg_ticks = sum as f64 / trials as f64;

    let missed_sum = misses.iter().fold(0u64, |sum, i| sum + (*i as u64));
    let missed_avg_ticks = missed_sum as f64 / misses.len() as f64;

    let threshold = avg_ticks * 1.31;
    let slows = times.iter().filter(|trial| **trial >= threshold as u16).count();

    println!("TTK with {} BGS specs: {:.4}", bgs, avg_ticks * 0.6);
    println!("  miss %: {}, TTK: {}", misses.len() * 100 / trials, missed_avg_ticks * 0.6);
    println!("  % slower than {:.4}: {}", threshold * 0.6, slows * 100 / trials);
}

fn main() {
    println!("Sim!");

    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).unwrap();
    let seed = u64::from_ne_bytes(buf);
    let mut rng_gen = randomize::PCG32::seed(seed, seed);

    simulate_n(10_000_000, 0, &mut rng_gen);
    simulate_n(10_000_000, 1, &mut rng_gen);
    simulate_n(10_000_000, 2, &mut rng_gen);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zero_def_reduction() {
        let mut boss = Boss::new(500, 180, 20, 120);
        boss.reduce_def(0);
        assert_eq!(boss.def_lvl, 180);
        assert_eq!(boss.mdr, 41277);
        assert_eq!(boss.hp, 500);
    }

    #[test]
    fn zero_def_slash_reduction() {
        let mut boss = Boss::new(500, 180, 40, 120);
        boss.reduce_def(0);
        assert_eq!(boss.def_lvl, 180);
        assert_eq!(boss.mdr, 51105);
        assert_eq!(boss.hp, 500);
    }

    #[test]
    fn small_def_reduction() {
        let mut boss = Boss::new(500, 180, 20, 120);
        boss.reduce_def(20);
        assert_eq!(boss.def_lvl, 160);
        assert_eq!(boss.mdr, 36909);
        assert_eq!(boss.hp, 480);
    }

    #[test]
    fn large_def_reduction() {
        let mut boss = Boss::new(500, 180, 20, 120);
        boss.reduce_def(80);
        assert_eq!(boss.def_lvl, 120);
        assert_eq!(boss.mdr, 28173);
        assert_eq!(boss.hp, 420);
    }
}
