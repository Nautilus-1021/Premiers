use std::{iter::repeat_with, sync::atomic::{AtomicU32, AtomicUsize, Ordering}, thread, time::Instant};
use primes_export::PrimesExport;

static COUNTER: AtomicU32 = AtomicU32::new(3);
static PRIMES_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let mut buf = repeat_with(|| AtomicU32::new(0)).take(1_000_000).collect::<Vec<_>>();

    let t1 = Instant::now();
    thread::scope(|th_scope| {
        for _idx in 0..12 {
            let _th_handle = th_scope.spawn(|| thread_main(&buf));
        }
    });

    let duration = t1.elapsed();

    buf.sort_unstable_by(|a, b| a.load(Ordering::Relaxed).cmp(&b.load(Ordering::Relaxed)));

    let mut expt = PrimesExport::new("rust + thread2.expt").unwrap();

    for el in buf.iter().map(|nmb| { nmb.load(Ordering::Relaxed) }) {
        expt.export_prime(el).unwrap();

        if el % 100 == 1 {
            print!("\x1b[2K\x1b[1G{}", el);
        }
    }
    println!();

    expt.close().unwrap();

    println!("Duration: {}s", duration.as_secs_f64());
    println!("Fin: {}", COUNTER.load(Ordering::Relaxed));
}

fn thread_main(primes: &Vec<AtomicU32>) {
    'outer: while PRIMES_COUNTER.load(Ordering::Relaxed) < 1_000_000 {
        let act_nmb = COUNTER.fetch_add(2, Ordering::SeqCst);
        let sqrt_nmb = f64::from(act_nmb).sqrt().round() as u32;

        for prime_idx in 0..PRIMES_COUNTER.load(Ordering::Relaxed) {
            let prime = primes[prime_idx].load(Ordering::Relaxed);
            if prime > sqrt_nmb {
                break;
            }

            if act_nmb % prime == 0 {
                continue 'outer;
            }
        }

        let prime_idx = PRIMES_COUNTER.fetch_add(1, Ordering::SeqCst);
        if prime_idx >= 1_000_000 { continue; }
        primes[prime_idx].store(act_nmb, Ordering::SeqCst);
    }
}