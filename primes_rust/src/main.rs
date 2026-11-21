use std::{
    sync::atomic::{AtomicU32, Ordering}, thread, time::{Duration, Instant}
};

mod atomic_vec;

fn main() {
    let limite = std::env::args().nth(1).and_then(|val| val.parse::<u32>().ok()).unwrap_or(10_000_000);

    let (premiers, (tt, reallocs)) = gen_premiers_multithread(limite);

    let x = limite as f64;
    let max_premiers = (x / (x.ln() - 1.1)).ceil() as usize;

    println!("Temps: {:.3}s, {} nombres premiers (MAX: {}), {} réallocations", tt.as_secs_f64(), premiers.len(), max_premiers, reallocs);
}

fn gen_premiers_simple(limite: usize) -> Vec<u32> {
    let mut lst = Vec::with_capacity(limite);
    lst.push(2);
    'outer: for nmb in (3..=limite as u32).step_by(2) {
        for divi in &lst {
            if nmb % divi == 0 {
                continue 'outer
            }
        }

        lst.push(nmb);
    }
    lst
}

fn gen_premiers_multithread(limite: u32) -> (Vec<u32>, (Duration, usize)) {
    let x = limite as f64;
    let primes_max = (x / (x.ln() - 1.1)).ceil() as usize;

    let primes = atomic_vec::AtomicVec::new(primes_max, 10);
    primes.push(2).unwrap();
    let cnt = AtomicU32::new(3);

    println!("Génération des nombres premiers");

    let t1 = Instant::now();

    let diviseurs = gen_premiers_simple((limite as f64).sqrt().ceil() as usize);

    thread::scope(|th_scope| {
        for _ in 0..*atomic_vec::PARALLELISM {
            th_scope.spawn(|| {
                let mut act_nmb;
                'outer: while cnt.load(Ordering::Relaxed) < limite {
                    act_nmb = cnt.fetch_add(2, Ordering::Relaxed);

                    for divi in &diviseurs {
                        if divi.pow(2) > act_nmb {
                            break;
                        }

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

    (primes, (tt, realloc_num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_simple() {
        assert_eq!(&gen_premiers_simple(20), &[2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn verif_algo() {
        assert_eq!(gen_premiers_simple(1_000_000), gen_premiers_multithread(1_000_000).0)
    }
}