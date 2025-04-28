use std::{env, fs::File, io::BufReader};

use primes_export::PrimesIter;

fn main() {
    let mut args = env::args();
    let f1 = args.nth(1).expect("Please specify a filename");
    let f2 = args.next().expect("Please specify two filenames");

    let file = BufReader::new(File::open(f1).unwrap());

    let pf1 = PrimesIter::new(file);

    let file = BufReader::new(File::open(f2).unwrap());

    let pf2 = PrimesIter::new(file);

    pf1.zip(pf2).enumerate().for_each(|(idx, (p1, p2))| {
        assert_eq!(p1, p2, "#{idx} did not match ({p1}, {p2})");
    });
}