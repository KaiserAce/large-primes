use num_bigint::{BigUint, RandBigInt};
use num_traits::{FromPrimitive, One, Pow, ToPrimitive, Zero};
use rand::{Rng, thread_rng};
use rayon::prelude::*;
use std::collections::HashSet;

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

#[derive(Clone, Copy, Debug)]
enum Prime {
    Random,
    Safe,
    Mersenne,
}

fn digits_to_bits(digits: usize) -> usize {
    ((digits as f64) * 3.32192809489).ceil() as usize
}

fn generate_n_digit_number(digits: usize) -> BigUint {
    let mut rng = thread_rng();
    let lower = BigUint::from(10u32).pow(digits as u32 - 1);
    let upper = BigUint::from(10u32).pow(digits as u32);

    rng.gen_biguint_range(&lower, &upper)
}

fn sieve_check(candidate: &BigUint) -> bool {
    if candidate % 2u32 == BigUint::zero() && candidate != &BigUint::from(2u32) {
        return false;
    }

    for &p in SMALL_PRIMES.iter().skip(1) {
        let bp = BigUint::from_u32(p).unwrap();
        if candidate % &bp == BigUint::zero() && candidate != &bp {
            return false;
        }
    }
    true
}

fn miller_rabin_check(candidate: &BigUint, k: u32) -> bool {
    if candidate <= &BigUint::one() {
        return false;
    }

    for &p in SMALL_PRIMES.iter() {
        let bp = BigUint::from_u32(p).unwrap();
        if candidate == &bp {
            return true;
        }
    }

    let mut d = candidate - BigUint::one();
    let mut s = 0;

    while &d % 2u32 == BigUint::zero() {
        d /= 2u32;
        s += 1;
    }

    let mut rng = thread_rng();
    'witness: for _ in 0..k {
        let a = rng.gen_biguint_range(&BigUint::from(2u32), &(candidate - 2u32));

        let mut x = a.modpow(&d, candidate);

        if x == BigUint::one() || x == candidate - BigUint::one() {
            continue 'witness;
        }

        for _ in 0..s - 1 {
            x = x.modpow(&BigUint::from(2u32), candidate);
            if x == candidate - BigUint::one() {
                continue 'witness;
            }
        }

        return false;
    }

    true
}

fn gen_rand_large_prime(digits: usize) -> BigUint {
    let parallel_threshhold = 100;
    let use_parallel = digits >= parallel_threshhold;

    if use_parallel {
        return gen_rand_large_prime_parallel(digits)
    }
    loop {
        let mut candidate = generate_n_digit_number(digits);

        if &candidate % 2u32 == BigUint::zero() {
            candidate += BigUint::one();
            if candidate >= BigUint::from(10u32).pow(digits as u32) {
                continue;
            }
        }

        if !sieve_check(&candidate) {
            continue;
        }

        if miller_rabin_check(&candidate, 20) {
            return candidate;
        }
    }
}

fn gen_rand_large_prime_parallel(digits: usize) -> BigUint {
    let num_candidates = num_cpus::get() * 2;
    
    loop {
        let candidates: Vec<BigUint> = (0..num_candidates)
            .map(|_| {
                let mut n = generate_n_digit_number(digits);
                if &n % 2u32 == BigUint::zero() {
                    n += BigUint::one();
                }
                n
            })
            .collect();
        
        let sieved_candidates: Vec<BigUint> = candidates
            .into_par_iter()
            .filter(|c| sieve_check(c))
            .collect();
        
        let prime_result = sieved_candidates
            .par_iter()
            .find_first(|c| miller_rabin_check(c, 20));
        
        if let Some(prime) = prime_result {
            return prime.clone();
        }
    }
}

fn gen_safe_prime(digits: usize) -> BigUint {
    loop {
        let q = gen_rand_large_prime(digits - 1);

        let p = &q * 2u32 + BigUint::one();

        let p_digits = p.to_string().len();
        if  p_digits != digits {
            continue;
        }

        if sieve_check(&p) && miller_rabin_check(&p, 20) {
            return p;
        }
    }
}

fn gen_mersenne_prime(digits: usize) -> BigUint {
    todo!()
}

fn gen_large_prime(prime_type: Prime, digits: usize) -> BigUint {
    match prime_type {
        Prime::Random => gen_rand_large_prime(digits),
        Prime::Safe => gen_safe_prime(digits),
        Prime::Mersenne => gen_mersenne_prime(digits),
    }
}

fn main() {
    println!("{}", gen_large_prime(Prime::Random, 100));
    println!("{}", gen_large_prime(Prime::Safe, 100));
}
