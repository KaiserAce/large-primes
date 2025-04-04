use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use std::process::Command;
use std::path::Path;
use std::fs;


use num_bigint::{BigUint, RandBigInt};
use num_prime::{nt_funcs::is_prime, Primality, PrimalityTestConfig};
use num_traits::{FromPrimitive, One, Pow, Zero};
use rand::thread_rng;
use rayon::prelude::*;

const SMALL_PRIMES: [u32; 201] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541, 547,
    557, 563, 569, 571, 577, 587, 593, 599, 601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659,
    661, 673, 677, 683, 691, 701, 709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797,
    809, 811, 821, 823, 827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929,
    937, 941, 947, 953, 967, 971, 977, 983, 991, 997, 1009, 1013, 1019, 1021, 1031, 1033, 1039,
    1049, 1051, 1061, 1063, 1069, 1087, 1091, 1093, 1097, 1103, 1109, 1117, 1123, 1129, 1151, 1153,
    1163, 1171, 1181, 1187, 1193, 1201, 1213, 1217, 1223, 1229,
];

fn probable_prime_check(candidate: &BigUint) -> bool {
    let config = PrimalityTestConfig::strict();
    match is_prime(candidate, Some(config)) {
        Primality::Yes => true,
        Primality::No => false,
        Primality::Probable(_) =>{
            true
        },
    }
}

fn deterministic_prime_check(candidate: &BigUint) -> bool {
    let script_content = format!("default(parisizemax, 12000000000) \nprint(isprime({}));\nquit;", candidate);
    let script_path = Path::new("temp_prime_check.gp");
    
    let _ = fs::write(script_path, script_content);

    let output = Command::new("gp")
        .arg("-q")
        .arg(script_path)
        .output()
        .expect("Failed to run PARI/GP");

    let _ = fs::remove_file(script_path);

    // println!("{}", String::from_utf8_lossy(&output.stdout));
    // println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    String::from_utf8_lossy(&output.stdout).trim() == "1"
}

fn generate_prime_candidate(digits: usize) -> BigUint {
    let lower = BigUint::from(10u32).pow(digits as u32 - 1);
    let upper = BigUint::from(10u32).pow(digits as u32);

    let mut rng = thread_rng();

    loop {
        let num = rng.gen_biguint_range(&lower, &upper);

        if &num % 2u32 == BigUint::zero() {
            let odd_num = num + BigUint::one();

            if odd_num < upper {
                return odd_num;
            } else {
                continue;
            }
        }

        return num;
    }
}

fn sieve_check(candidate: &BigUint) -> bool {
    for &p in SMALL_PRIMES.iter().skip(1) {
        let bp = BigUint::from_u32(p).unwrap();
        if candidate % &bp == BigUint::zero() && candidate != &bp {
            return false;
        }
    }
    true
}

fn gen_rand_large_prime(digits: usize) -> BigUint {
    let parallel_threshhold = 50;

    if digits >= parallel_threshhold {
        gen_rand_large_prime_parallel(digits)
    } else {
        gen_rand_large_prime_sequential(digits)
    }
}

fn gen_rand_large_prime_sequential(digits: usize) -> BigUint {
    loop {
        let candidate = generate_prime_candidate(digits);

        if !sieve_check(&candidate) {
            continue;
        }

        if probable_prime_check(&candidate) {
            if deterministic_prime_check(&candidate) {
                return candidate;
            }
        }
    }
}

fn gen_rand_large_prime_parallel(digits: usize) -> BigUint {
    let cpus = num_cpus::get();
    let num_candidates = cpus * 4;
    let found = Arc::new(AtomicBool::new(false));

    loop {
        let candidates: Vec<BigUint> = (0..num_candidates)
            .into_par_iter()
            .map(|_| generate_prime_candidate(digits))
            .collect();

        let sieved_candidates: Vec<BigUint> = candidates
            .into_par_iter()
            .filter(|c| sieve_check(c))
            .collect();

        let found_arc = Arc::clone(&found);
        let prime_result = sieved_candidates.par_iter().find_map_first(|c| {
            if found_arc.load(Ordering::Relaxed) {
                return None;
            }

            if probable_prime_check(c) {
                if deterministic_prime_check(c) {
                    found_arc.store(true, Ordering::Relaxed);
                    Some(c.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(prime) = prime_result {
            found.store(false, Ordering::Relaxed);
            return prime.clone();
        }
        found.store(false, Ordering::Relaxed);
    }
}

fn main() {
    let digits_arr: Vec<usize> = vec![10, 50, 100, 300, 800, 1000, 1500];

    for digits in digits_arr {

        let start = Instant::now();
        let test = gen_rand_large_prime(digits);
        let duration = start.elapsed();
        println!("----------------------------------------");
        println!("{} digit prime:", digits);
        println!("{}", test);
        println!("Time elapsed: {:.2?}", duration);
        println!("----------------------------------------");

    }
}
