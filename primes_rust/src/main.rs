use std::{sync::atomic::{AtomicU32, Ordering}, thread, time::Instant};

mod atomic_vec;

const NMB_PREMIERS: u32 = 10_000_000;

fn main() {
    let x = NMB_PREMIERS as f64;
    let prime_estimate = (x / x.ln()).round() as usize;
    let primes_max = (x / (x.ln() - 1.1)).ceil() as usize;

    let primes = atomic_vec::AtomicVec::new(prime_estimate, primes_max - prime_estimate + 3);
    let cnt = AtomicU32::new(3);

    println!("Génération des nombres premiers");

    let t1 = Instant::now();
    thread::scope(|th_scope| {
        for _ in 0..*atomic_vec::PARALLELISM {
            th_scope.spawn(|| {
                let mut act_nmb;
                'outer: while cnt.load(Ordering::Relaxed) < NMB_PREMIERS {
                    act_nmb = cnt.fetch_add(2, Ordering::SeqCst);
    
                    let sqrt_nmb = (act_nmb as f64).sqrt() as u32;
                    for divi in (3..=sqrt_nmb).step_by(2) {
                        if act_nmb % divi == 0 {
                            continue 'outer;
                        }
                    }

                    primes.push(act_nmb).unwrap();
                }
            });
        }
    });

    let tt = t1.elapsed();

    let realloc_num = primes.get_realloc_nmb();

    let mut primes = primes.iter().collect::<Vec<u32>>();

    println!("Tri des nombres");

    primes.sort_unstable();

    println!("Vérification");

    let mut pr_cnt = 0;
    let mut cnt = 3;
    while pr_cnt < primes.len() {
        let chk_pr = primes[pr_cnt];
        let (verif, divi) = verif_premier(cnt);

        if !verif {
            if cnt == chk_pr {
                println!("{cnt} n'est pas premier (divi: {divi})");
                pr_cnt += 1;
            }

            cnt += 2;
            continue;
        }

        if cnt < chk_pr {
            println!("{cnt} passé");
        } else if cnt > chk_pr {
            println!("{cnt}, {chk_pr}, {pr_cnt}");
            panic!("bug");
        } else if cnt == chk_pr {
            pr_cnt += 1;
        }

        cnt += 2;
    }

    println!("Temps: {:.3}s, {pr_cnt} nombres premiers, {realloc_num} réallocations", tt.as_secs_f64());
}

fn verif_premier(nmb: u32) -> (bool, u32) {
    if nmb % 2 == 0 {
        return (false, 2)
    }

    let sqrt_nmb = f64::from(nmb).sqrt().ceil() as u32;
    for chk in (3..=sqrt_nmb).step_by(2) {
        if nmb % chk == 0 {
            return (false, chk)
        }
    }

    (true, 0)
}