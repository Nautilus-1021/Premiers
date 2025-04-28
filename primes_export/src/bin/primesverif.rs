use std::{env, fs::File, io::BufReader};

use primes_export::PrimesIter;

fn main() {
    let filename = env::args().nth(1).expect("Please specify a filename");

    let file = BufReader::new(File::open(filename).unwrap());

    let primes_iter = PrimesIter::new(file);

    let mut counter = 3;
    for prime_tc in primes_iter {
        if prime_tc % 100 == 1 {
            print!("\x1b[2K\x1b[1G{}", prime_tc);
        }

        while !is_prime(counter) { counter += 2; }

        if prime_tc != counter {
            println!("\nLes nombres ne correspondent pas. Attendu: {counter}, Trouvé: {prime_tc}");
            break;
        }

        counter += 2;
    }
}

fn is_prime(tc: u32) -> bool {
    if tc % 2 == 0 { return false }

    let tc_sq = f64::from(tc).sqrt().round() as u32;

    for divi in (3..=tc_sq).step_by(2) {
        if tc % divi == 0 {
            return false
        }
    }

    true
}