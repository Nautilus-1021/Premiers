use std::{env, fs::File, io::BufReader};

use primes_export::PrimesIter;

fn main() {
    let filename = env::args().nth(1).expect("Please specify a filename");

    let file = BufReader::new(File::open(&filename).unwrap());

    let primes_iter = PrimesIter::new(file);

    println!("Loading {filename}");

    for pr in primes_iter {
        print!("{pr} ");
    }
    println!();
}