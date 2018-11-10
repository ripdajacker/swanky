use num::integer::Integer;
use num::bigint::BigInt;
use num::{ToPrimitive, Zero, One, Signed};
use num_traits::pow::pow;
use util::IterToVec;

pub fn digits_per_u128(modulus: u16) -> usize {
    (128.0 / (modulus as f64).log2().ceil()).floor() as usize
}

pub fn base_q_add(xs: &[u16], ys: &[u16], q: u16) -> Vec<u16> {
    if ys.len() > xs.len() {
        return base_q_add(ys, xs, q);
    }
    let mut ret = xs.to_vec();
    base_q_add_eq(&mut ret, ys, q);
    ret
}

pub fn base_q_add_eq(xs: &mut [u16], ys: &[u16], q: u16)
{
    debug_assert!(
        xs.len() >= ys.len(),
        "q={} xs.len()={} ys.len()={} xs={:?} ys={:?}",
        q, xs.len(), ys.len(), xs, ys
    );

    let mut c = 0;
    let mut i = 0;

    while i < ys.len() {
        xs[i] += ys[i] + c;
        c = 0;
        if xs[i] >= q {
            xs[i] -= q;
            c = 1;
        }
        i += 1;
    }

    // continue the carrying if possible
    while i < xs.len() {
        xs[i] += c;
        if xs[i] >= q {
            xs[i] -= q;
            // c = 1
        } else {
            // c = 0
            break;
        }
        i += 1;
    }
}

pub fn as_base_q(x: u128, q: u16) -> Vec<u16> {
    let n = digits_per_u128(q);
    println!("q={} n={}", q, n);
    assert!(BigInt::from(x) < pow(BigInt::from(q), n), "q={}", q);
    let ms = std::iter::repeat(q).take(n).to_vec();
    as_mixed_radix(x, &ms)
}

pub fn as_mixed_radix(x: u128, ms: &[u16]) -> Vec<u16> {
    let mut ds = Vec::with_capacity(ms.len());
    let mut x = x;

    for i in 0..ms.len() {
        if x >= ms[i] as u128 {
            let d = x % ms[i] as u128;
            x = (x - d) / ms[i] as u128;
            ds.push(d as u16);
        } else {
            ds.push(x as u16);
            break;
        }
    }
    ds
}

pub fn padded_mixed_radix(x: u128, ms: &[u16]) -> Vec<u16> {
    let mut ds = as_mixed_radix(x,ms);
    while ds.len() < ms.len() {
        ds.push(0);
    }
    ds
}

fn u16_from_bigint(bi: &BigInt) -> u16 {
    let (_,bs) = bi.to_bytes_le();
    let mut x = 0;
    x += bs[0] as u16;
    x += (bs[1] as u16) << 16;
    x
}

pub fn as_mixed_radix_bigint(x: &BigInt, ms: &[u16]) -> Vec<u16> {
    let mut ds = Vec::with_capacity(ms.len());
    let mut x = x.clone();

    for i in 0..ms.len() {
        let m = BigInt::from(ms[i]);
        if &x >= &m {
            let d = &x % &m;
            x = (&x - &d) / &m;
            ds.push(u16_from_bigint(&d));
        } else {
            ds.push(u16_from_bigint(&x));
            break;
        }
    }
    ds
}

pub fn padded_mixed_radix_bigint(x: &BigInt, ms: &[u16]) -> Vec<u16> {
    let mut ds = as_mixed_radix_bigint(x,ms);
    while ds.len() < ms.len() {
        ds.push(0);
    }
    ds
}

pub fn from_base_q(ds: &[u16], q: u16) -> u128 {
    let q = q as u128;
    let mut x: u128 = 0;
    for &d in ds.iter().rev() {
        let (xp,overflow) = x.overflowing_mul(q);
        assert_eq!(overflow, false, "overflow!!!! x={}", x);
        // x = x * q + d as u128;
        x = xp + d as u128;
    }
    x
}

pub fn padded_base_q(x: u128, q: u16, n: usize) -> Vec<u16> {
    let ms = std::iter::repeat(q).take(n).collect::<Vec<_>>();
    padded_mixed_radix(x, &ms)
}

pub fn padded_base_q_128(x: u128, q: u16) -> Vec<u16> {
    let n  = digits_per_u128(q);
    let ms = std::iter::repeat(q).take(n).collect::<Vec<_>>();
    padded_mixed_radix(x, &ms)
}

pub fn u128_to_bits(x: u128, n: usize) -> Vec<u16> {
    to_bits(x,n)
}

pub fn to_bits<I,B>(x: I, n: usize) -> Vec<B>
    where I: Clone + std::ops::Rem + std::ops::SubAssign + std::ops::DivAssign + One +
             std::ops::BitAnd<Output=I> + std::convert::From<u16>,
          B: std::convert::TryFrom<I>
{
    let mut bits = Vec::with_capacity(n);
    let mut y = x;
    for _ in 0..n {
        let b = y.clone() & I::one();
        match B::try_from(b.clone()) {
            Ok(b) => bits.push(b),
            _ => panic!(),
        }
        y -= b;
        y /= I::from(2);
    }
    bits
}

pub fn u128_from_bits(bs: &[u16]) -> u128 {
    let mut x = 0;
    for &b in bs.iter().skip(1).rev() {
        x += b as u128;
        x *= 2;
    }
    x += bs[0] as u128;
    x
}

// only factor using the above primes- we only support composites with small
// prime factors in the high-level circuit representation
pub fn factor(inp: u128) -> Vec<u16> {
    let mut x = inp;
    let mut fs = Vec::new();
    for &p in PRIMES.iter() {
        let q = p as u128;
        if x % q == 0 {
            fs.push(p);
            x /= q;
        }
    }
    if x != 1 {
        panic!("can only factor numbers with unique prime factors");
    }
    fs
}

pub fn crt(ps: &[u16], x: u128) -> Vec<u16> {
    ps.iter().map(|&p| {
        (x % p as u128) as u16
    }).collect()
}

pub fn crt_inv(ps: &[u16], xs: &[u16]) -> u128 {
    let mut ret = BigInt::zero();

    let M = ps.iter().fold(BigInt::one(), |acc, &x| BigInt::from(x) * acc );

    for (&p, &a) in ps.iter().zip(xs.iter()) {
        let p = BigInt::from(p);
        let q = &M / &p;
        ret += BigInt::from(a) * inv_ref(&q,&p) * q;
        ret %= &M;
    }

    ret.to_u128().unwrap()
}

pub fn inv_ref<T: Clone + Integer + Signed>(inp_a: &T, inp_b: &T) -> T {
    let mut a = inp_a.clone();
    let mut b = inp_b.clone();
    let mut q;
    let mut tmp;

    let (mut x0, mut x1) = (T::zero(), T::one());

    if b == T::one() {
        return T::one();
    }

    while a > T::one() {
        q = a.clone() / b.clone();

        // a, b = b, a%b
        tmp = b.clone();
        b = a.clone() % b.clone();
        a = tmp;

        tmp = x0.clone();
        x0 = x1.clone() - q.clone() * x0.clone();
        x1 = tmp.clone();
    }

    if x1 < T::zero() {
        x1 = x1 + inp_b.clone();
    }

    x1
}

pub fn inv<T: Copy + Integer + Signed>(a: T, m: T) -> T {
    inv_ref(&a, &m)
}

pub const NPRIMES: usize = 29;

pub const PRIMES: [u16;29] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
    73, 79, 83, 89, 97, 101, 103, 107, 109
];

pub const PRIMES_SKIP_2: [u16;29] = [
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
    73, 79, 83, 89, 97, 101, 103, 107, 109, 113
];

pub fn modulus_with_width(nbits: u32) -> u128 {
    base_modulus_with_width(nbits, &PRIMES)
}

pub fn modulus_with_width_skip2(nbits: u32) -> u128 {
    base_modulus_with_width(nbits, &PRIMES_SKIP_2)
}

pub fn base_modulus_with_width(nbits: u32, ps: &[u16]) -> u128 {
    let mut res = 1;
    let mut i = 0;
    loop {
        res *= u128::from(ps[i]);
        if (res >> nbits) > 0 {
            break;
        }
        i += 1;
        assert!(i < ps.len());
    }
    res
}


pub fn product(xs: &[u16]) -> u128 {
    xs.iter().fold(1, |acc, &x| acc * x as u128)
}

pub const PRIMITIVE_ROOTS: [u16;29] = [
    2, 2, 3, 2, 2, 3, 2, 5, 2, 3, 2, 6, 3, 5, 2, 2, 2, 2, 7, 5, 3, 2, 3, 5, 2,
    5, 2, 6, 3
];

// note that the first element is meaningless since dlog(0) is undefined
pub fn dlog_truth_table(modulus: u16) -> Vec<u16> {
    match modulus {
        2 => vec![0, 0],

        3 => vec![0, 0,1],

        5 => vec![0, 0,1,3,2],

        7 => vec![0, 0,2,1,4,5,3],

        11 => vec![0, 0,1,8,2,4,9,7,3,6,5],

        13 => vec![0, 0,1,4,2,9,5,11,3,8,10,7,6],

        17 => vec![0, 0, 14, 1, 12, 5, 15, 11, 10, 2, 3, 7, 13, 4, 9, 6, 8],

        19 => vec![0, 0, 1, 13, 2, 16, 14, 6, 3, 8, 17, 12, 15, 5, 7, 11, 4, 10,
           9],

        23 => vec![0, 0, 2, 16, 4, 1, 18, 19, 6, 10, 3, 9, 20, 14, 21, 17, 8, 7,
           12, 15, 5, 13, 11],

        29 => vec![0, 0, 1, 5, 2, 22, 6, 12, 3, 10, 23, 25, 7, 18, 13, 27, 4,
           21, 11, 9, 24, 17, 26, 20, 8, 16, 19, 15, 14],

        31 => vec![0, 0, 24, 1, 18, 20, 25, 28, 12, 2, 14, 23, 19, 11, 22, 21,
           6, 7, 26, 4, 8, 29, 17, 27, 13, 10, 5, 3, 16, 9, 15],

        37 => vec![0, 0, 1, 26, 2, 23, 27, 32, 3, 16, 24, 30, 28, 11, 33, 13, 4,
           7, 17, 35, 25, 22, 31, 15, 29, 10, 12, 6, 34, 21, 14, 9, 5, 20, 8,
           19, 18],

        41 => vec![0, 0, 26, 15, 12, 22, 1, 39, 38, 30, 8, 3, 27, 31, 25, 37,
           24, 33, 16, 9, 34, 14, 29, 36, 13, 4, 17, 5, 11, 7, 23, 28, 10, 18,
           19, 21, 2, 32, 35, 6, 20],

        43 => vec![0, 0, 27, 1, 12, 25, 28, 35, 39, 2, 10, 30, 13, 32, 20, 26,
           24, 38, 29, 19, 37, 36, 15, 16, 40, 8, 17, 3, 5, 41, 11, 34, 9, 31,
           23, 18, 14, 7, 4, 33, 22, 6, 21],

        47 => vec![0, 0, 18, 20, 36, 1, 38, 32, 8, 40, 19, 7, 10, 11, 4, 21, 26,
           16, 12, 45, 37, 6, 25, 5, 28, 2, 29, 14, 22, 35, 39, 3, 44, 27, 34,
           33, 30, 42, 17, 31, 9, 15, 24, 13, 43, 41, 23],

        53 => vec![0, 0, 1, 17, 2, 47, 18, 14, 3, 34, 48, 6, 19, 24, 15, 12, 4,
           10, 35, 37, 49, 31, 7, 39, 20, 42, 25, 51, 16, 46, 13, 33, 5, 23, 11,
           9, 36, 30, 38, 41, 50, 45, 32, 22, 8, 29, 40, 44, 21, 28, 43, 27,
           26],

        59 => vec![0, 0, 1, 50, 2, 6, 51, 18, 3, 42, 7, 25, 52, 45, 19, 56, 4,
           40, 43, 38, 8, 10, 26, 15, 53, 12, 46, 34, 20, 28, 57, 49, 5, 17, 41,
           24, 44, 55, 39, 37, 9, 14, 11, 33, 27, 48, 16, 23, 54, 36, 13, 32,
           47, 22, 35, 31, 21, 30, 29],

        61 => vec![0, 0, 1, 6, 2, 22, 7, 49, 3, 12, 23, 15, 8, 40, 50, 28, 4,
           47, 13, 26, 24, 55, 16, 57, 9, 44, 41, 18, 51, 35, 29, 59, 5, 21, 48,
           11, 14, 39, 27, 46, 25, 54, 56, 43, 17, 34, 58, 20, 10, 38, 45, 53,
           42, 33, 19, 37, 52, 32, 36, 31, 30],

        67 => vec![0, 0, 1, 39, 2, 15, 40, 23, 3, 12, 16, 59, 41, 19, 24, 54, 4,
           64, 13, 10, 17, 62, 60, 28, 42, 30, 20, 51, 25, 44, 55, 47, 5, 32,
           65, 38, 14, 22, 11, 58, 18, 53, 63, 9, 61, 27, 29, 50, 43, 46, 31,
           37, 21, 57, 52, 8, 26, 49, 45, 36, 56, 7, 48, 35, 6, 34, 33],

        71 => vec![0, 0, 6, 26, 12, 28, 32, 1, 18, 52, 34, 31, 38, 39, 7, 54,
           24, 49, 58, 16, 40, 27, 37, 15, 44, 56, 45, 8, 13, 68, 60, 11, 30,
           57, 55, 29, 64, 20, 22, 65, 46, 25, 33, 48, 43, 10, 21, 9, 50, 2, 62,
           5, 51, 23, 14, 59, 19, 42, 4, 3, 66, 69, 17, 53, 36, 67, 63, 47, 61,
           41, 35],

        73 => vec![0, 0, 8, 6, 16, 1, 14, 33, 24, 12, 9, 55, 22, 59, 41, 7, 32,
           21, 20, 62, 17, 39, 63, 46, 30, 2, 67, 18, 49, 35, 15, 11, 40, 61,
           29, 34, 28, 64, 70, 65, 25, 4, 47, 51, 71, 13, 54, 31, 38, 66, 10,
           27, 3, 53, 26, 56, 57, 68, 43, 5, 23, 58, 19, 45, 48, 60, 69, 50, 37,
           52, 42, 44, 36],

        79 => vec![0, 0, 4, 1, 8, 62, 5, 53, 12, 2, 66, 68, 9, 34, 57, 63, 16,
           21, 6, 32, 70, 54, 72, 26, 13, 46, 38, 3, 61, 11, 67, 56, 20, 69, 25,
           37, 10, 19, 36, 35, 74, 75, 58, 49, 76, 64, 30, 59, 17, 28, 50, 22,
           42, 77, 7, 52, 65, 33, 15, 31, 71, 45, 60, 55, 24, 18, 73, 48, 29,
           27, 41, 51, 14, 44, 23, 47, 40, 43, 39],

        83 => vec![0, 0, 1, 72, 2, 27, 73, 8, 3, 62, 28, 24, 74, 77, 9, 17, 4,
           56, 63, 47, 29, 80, 25, 60, 75, 54, 78, 52, 10, 12, 18, 38, 5, 14,
           57, 35, 64, 20, 48, 67, 30, 40, 81, 71, 26, 7, 61, 23, 76, 16, 55,
           46, 79, 59, 53, 51, 11, 37, 13, 34, 19, 66, 39, 70, 6, 22, 15, 45,
           58, 50, 36, 33, 65, 69, 21, 44, 49, 32, 68, 43, 31, 42, 41],

        89 => vec![0, 0, 16, 1, 32, 70, 17, 81, 48, 2, 86, 84, 33, 23, 9, 71,
           64, 6, 18, 35, 14, 82, 12, 57, 49, 52, 39, 3, 25, 59, 87, 31, 80, 85,
           22, 63, 34, 11, 51, 24, 30, 21, 10, 29, 28, 72, 73, 54, 65, 74, 68,
           7, 55, 78, 19, 66, 41, 36, 75, 43, 15, 69, 47, 83, 8, 5, 13, 56, 38,
           58, 79, 62, 50, 20, 27, 53, 67, 77, 40, 42, 46, 4, 37, 61, 26, 76,
           45, 60, 44],

        97 => vec![0, 0, 34, 70, 68, 1, 8, 31, 6, 44, 35, 86, 42, 25, 65, 71,
           40, 89, 78, 81, 69, 5, 24, 77, 76, 2, 59, 18, 3, 13, 9, 46, 74, 60,
           27, 32, 16, 91, 19, 95, 7, 85, 39, 4, 58, 45, 15, 84, 14, 62, 36, 63,
           93, 10, 52, 87, 37, 55, 47, 67, 43, 64, 80, 75, 12, 26, 94, 57, 61,
           51, 66, 11, 50, 28, 29, 72, 53, 21, 33, 30, 41, 88, 23, 17, 73, 90,
           38, 83, 92, 54, 79, 56, 49, 20, 22, 82, 48],

        101 => vec![0, 0, 1, 69, 2, 24, 70, 9, 3, 38, 25, 13, 71, 66, 10, 93, 4,
            30, 39, 96, 26, 78, 14, 86, 72, 48, 67, 7, 11, 91, 94, 84, 5, 82,
            31, 33, 40, 56, 97, 35, 27, 45, 79, 42, 15, 62, 87, 58, 73, 18, 49,
            99, 68, 23, 8, 37, 12, 65, 92, 29, 95, 77, 85, 47, 6, 90, 83, 81,
            32, 55, 34, 44, 41, 61, 57, 17, 98, 22, 36, 64, 28, 76, 46, 89, 80,
            54, 43, 60, 16, 21, 63, 75, 88, 53, 59, 20, 74, 52, 19, 51, 50],

        103 => vec![0, 0, 44, 39, 88, 1, 83, 4, 30, 78, 45, 61, 25, 72, 48, 40,
            74, 70, 20, 80, 89, 43, 3, 24, 69, 2, 14, 15, 92, 86, 84, 57, 16,
            100, 12, 5, 64, 93, 22, 9, 31, 50, 87, 77, 47, 79, 68, 85, 11, 8,
            46, 7, 58, 97, 59, 62, 34, 17, 28, 98, 26, 36, 101, 82, 60, 73, 42,
            13, 56, 63, 49, 67, 6, 33, 35, 41, 66, 65, 53, 18, 75, 54, 94, 38,
            29, 71, 19, 23, 91, 99, 21, 76, 10, 96, 27, 81, 55, 32, 52, 37, 90,
            95, 51],

        107 => vec![0, 0, 1, 70, 2, 47, 71, 43, 3, 34, 48, 22, 72, 14, 44, 11,
            4, 29, 35, 78, 49, 7, 23, 62, 73, 94, 15, 104, 45, 32, 12, 27, 5,
            92, 30, 90, 36, 38, 79, 84, 50, 40, 8, 59, 24, 81, 63, 66, 74, 86,
            95, 99, 16, 52, 105, 69, 46, 42, 33, 21, 13, 10, 28, 77, 6, 61, 93,
            103, 31, 26, 91, 89, 37, 83, 39, 58, 80, 65, 85, 98, 51, 68, 41, 20,
            9, 76, 60, 102, 25, 88, 82, 57, 64, 97, 67, 19, 75, 101, 87, 56, 96,
            18, 100, 55, 17, 54, 53],

        109 => vec![0, 0, 57, 52, 6, 76, 1, 40, 63, 104, 25, 83, 58, 67, 97, 20,
            12, 93, 53, 75, 82, 92, 32, 33, 7, 44, 16, 48, 46, 34, 77, 14, 69,
            27, 42, 8, 2, 5, 24, 11, 31, 45, 41, 30, 89, 72, 90, 17, 64, 80,
            101, 37, 73, 49, 105, 51, 103, 19, 91, 47, 26, 10, 71, 36, 18, 35,
            84, 95, 99, 85, 65, 78, 59, 56, 62, 96, 81, 15, 68, 23, 88, 100,
            102, 70, 98, 61, 87, 86, 38, 28, 21, 107, 39, 66, 74, 43, 13, 4, 29,
            79, 50, 9, 94, 55, 22, 60, 106, 3, 54],

        113 => vec![0, 0, 12, 1, 24, 83, 13, 8, 36, 2, 95, 74, 25, 22, 20, 84,
            48, 5, 14, 99, 107, 9, 86, 41, 37, 54, 34, 3, 32, 89, 96, 50, 60,
            75, 17, 91, 26, 67, 111, 23, 7, 94, 21, 47, 98, 85, 53, 31, 49, 16,
            66, 6, 46, 52, 15, 45, 44, 100, 101, 71, 108, 102, 62, 10, 72, 105,
            87, 109, 29, 42, 103, 77, 38, 63, 79, 55, 11, 82, 35, 73, 19, 4,
            106, 40, 33, 88, 59, 90, 110, 93, 97, 30, 65, 51, 43, 70, 61, 104,
            28, 76, 78, 81, 18, 39, 58, 92, 64, 69, 27, 80, 57, 68, 56],

        p => panic!("unknown modulus: {}", p)
    }
}

pub fn powm(inp: u16, pow: u16, modulus: u16) -> u16 {
    let mut x = inp as u16;
    let mut z = 1;
    let mut n = pow;
    while n > 0 {
        if n % 2 == 0 {
            x = x.pow(2) % modulus as u16;
            n /= 2;
        } else {
            z = x * z % modulus as u16;
            n -= 1;
        }
    }
    z as u16
}

pub fn exp_truth_table(modulus: u16) -> Vec<u16> {
    match modulus {
        2 => vec![1, 1],

        3 => vec![1, 2, 1],

        5 => vec![1, 2, 4, 3, 1],

        7 => vec![1, 3, 2, 6, 4, 5, 1],

        11 => vec![1, 2, 4, 8, 5, 10, 9, 7, 3, 6, 1],

        13 => vec![1, 2, 4, 8, 3, 6, 12, 11, 9, 5, 10, 7, 1],

        17 => vec![1, 3, 9, 10, 13, 5, 15, 11, 16, 14, 8, 7, 4, 12, 2, 6, 1],

        19 => vec![1, 2, 4, 8, 16, 13, 7, 14, 9, 18, 17, 15, 11, 3, 6, 12, 5,
           10, 1],

        23 => vec![1, 5, 2, 10, 4, 20, 8, 17, 16, 11, 9, 22, 18, 21, 13, 19, 3,
           15, 6, 7, 12, 14, 1],

        29 => vec![1, 2, 4, 8, 16, 3, 6, 12, 24, 19, 9, 18, 7, 14, 28, 27, 25,
           21, 13, 26, 23, 17, 5, 10, 20, 11, 22, 15, 1],

        31 => vec![1, 3, 9, 27, 19, 26, 16, 17, 20, 29, 25, 13, 8, 24, 10, 30,
           28, 22, 4, 12, 5, 15, 14, 11, 2, 6, 18, 23, 7, 21, 1],

        37 => vec![1, 2, 4, 8, 16, 32, 27, 17, 34, 31, 25, 13, 26, 15, 30, 23,
           9, 18, 36, 35, 33, 29, 21, 5, 10, 20, 3, 6, 12, 24, 11, 22, 7, 14,
           28, 19, 1],

        41 => vec![1, 6, 36, 11, 25, 27, 39, 29, 10, 19, 32, 28, 4, 24, 21, 3,
           18, 26, 33, 34, 40, 35, 5, 30, 16, 14, 2, 12, 31, 22, 9, 13, 37, 17,
           20, 38, 23, 15, 8, 7, 1],

        43 => vec![1, 3, 9, 27, 38, 28, 41, 37, 25, 32, 10, 30, 4, 12, 36, 22,
           23, 26, 35, 19, 14, 42, 40, 34, 16, 5, 15, 2, 6, 18, 11, 33, 13, 39,
           31, 7, 21, 20, 17, 8, 24, 29, 1],

        47 => vec![1, 5, 25, 31, 14, 23, 21, 11, 8, 40, 12, 13, 18, 43, 27, 41,
           17, 38, 2, 10, 3, 15, 28, 46, 42, 22, 16, 33, 24, 26, 36, 39, 7, 35,
           34, 29, 4, 20, 6, 30, 9, 45, 37, 44, 32, 19, 1],

        53 => vec![1, 2, 4, 8, 16, 32, 11, 22, 44, 35, 17, 34, 15, 30, 7, 14,
           28, 3, 6, 12, 24, 48, 43, 33, 13, 26, 52, 51, 49, 45, 37, 21, 42, 31,
           9, 18, 36, 19, 38, 23, 46, 39, 25, 50, 47, 41, 29, 5, 10, 20, 40, 27,
           1],

        59 => vec![1, 2, 4, 8, 16, 32, 5, 10, 20, 40, 21, 42, 25, 50, 41, 23,
           46, 33, 7, 14, 28, 56, 53, 47, 35, 11, 22, 44, 29, 58, 57, 55, 51,
           43, 27, 54, 49, 39, 19, 38, 17, 34, 9, 18, 36, 13, 26, 52, 45, 31, 3,
           6, 12, 24, 48, 37, 15, 30, 1],

        61 => vec![1, 2, 4, 8, 16, 32, 3, 6, 12, 24, 48, 35, 9, 18, 36, 11, 22,
           44, 27, 54, 47, 33, 5, 10, 20, 40, 19, 38, 15, 30, 60, 59, 57, 53,
           45, 29, 58, 55, 49, 37, 13, 26, 52, 43, 25, 50, 39, 17, 34, 7, 14,
           28, 56, 51, 41, 21, 42, 23, 46, 31, 1],

        67 => vec![1, 2, 4, 8, 16, 32, 64, 61, 55, 43, 19, 38, 9, 18, 36, 5, 10,
           20, 40, 13, 26, 52, 37, 7, 14, 28, 56, 45, 23, 46, 25, 50, 33, 66,
           65, 63, 59, 51, 35, 3, 6, 12, 24, 48, 29, 58, 49, 31, 62, 57, 47, 27,
           54, 41, 15, 30, 60, 53, 39, 11, 22, 44, 21, 42, 17, 34, 1],

        71 => vec![1, 7, 49, 59, 58, 51, 2, 14, 27, 47, 45, 31, 4, 28, 54, 23,
           19, 62, 8, 56, 37, 46, 38, 53, 16, 41, 3, 21, 5, 35, 32, 11, 6, 42,
           10, 70, 64, 22, 12, 13, 20, 69, 57, 44, 24, 26, 40, 67, 43, 17, 48,
           52, 9, 63, 15, 34, 25, 33, 18, 55, 30, 68, 50, 66, 36, 39, 60, 65,
           29, 61, 1],

        73 => vec![1, 5, 25, 52, 41, 59, 3, 15, 2, 10, 50, 31, 9, 45, 6, 30, 4,
           20, 27, 62, 18, 17, 12, 60, 8, 40, 54, 51, 36, 34, 24, 47, 16, 7, 35,
           29, 72, 68, 48, 21, 32, 14, 70, 58, 71, 63, 23, 42, 64, 28, 67, 43,
           69, 53, 46, 11, 55, 56, 61, 13, 65, 33, 19, 22, 37, 39, 49, 26, 57,
           66, 38, 44, 1],

        79 => vec![1, 3, 9, 27, 2, 6, 18, 54, 4, 12, 36, 29, 8, 24, 72, 58, 16,
           48, 65, 37, 32, 17, 51, 74, 64, 34, 23, 69, 49, 68, 46, 59, 19, 57,
           13, 39, 38, 35, 26, 78, 76, 70, 52, 77, 73, 61, 25, 75, 67, 43, 50,
           71, 55, 7, 21, 63, 31, 14, 42, 47, 62, 28, 5, 15, 45, 56, 10, 30, 11,
           33, 20, 60, 22, 66, 40, 41, 44, 53, 1],

        83 => vec![1, 2, 4, 8, 16, 32, 64, 45, 7, 14, 28, 56, 29, 58, 33, 66,
           49, 15, 30, 60, 37, 74, 65, 47, 11, 22, 44, 5, 10, 20, 40, 80, 77,
           71, 59, 35, 70, 57, 31, 62, 41, 82, 81, 79, 75, 67, 51, 19, 38, 76,
           69, 55, 27, 54, 25, 50, 17, 34, 68, 53, 23, 46, 9, 18, 36, 72, 61,
           39, 78, 73, 63, 43, 3, 6, 12, 24, 48, 13, 26, 52, 21, 42, 1],

        89 => vec![1, 3, 9, 27, 81, 65, 17, 51, 64, 14, 42, 37, 22, 66, 20, 60,
           2, 6, 18, 54, 73, 41, 34, 13, 39, 28, 84, 74, 44, 43, 40, 31, 4, 12,
           36, 19, 57, 82, 68, 26, 78, 56, 79, 59, 88, 86, 80, 62, 8, 24, 72,
           38, 25, 75, 47, 52, 67, 23, 69, 29, 87, 83, 71, 35, 16, 48, 55, 76,
           50, 61, 5, 15, 45, 46, 49, 58, 85, 77, 53, 70, 32, 7, 21, 63, 11, 33,
           10, 30, 1],

        97 => vec![1, 5, 25, 28, 43, 21, 8, 40, 6, 30, 53, 71, 64, 29, 48, 46,
           36, 83, 27, 38, 93, 77, 94, 82, 22, 13, 65, 34, 73, 74, 79, 7, 35,
           78, 2, 10, 50, 56, 86, 42, 16, 80, 12, 60, 9, 45, 31, 58, 96, 92, 72,
           69, 54, 76, 89, 57, 91, 67, 44, 26, 33, 68, 49, 51, 61, 14, 70, 59,
           4, 20, 3, 15, 75, 84, 32, 63, 24, 23, 18, 90, 62, 19, 95, 87, 47, 41,
           11, 55, 81, 17, 85, 37, 88, 52, 66, 39, 1],

        101 => vec![1, 2, 4, 8, 16, 32, 64, 27, 54, 7, 14, 28, 56, 11, 22, 44,
            88, 75, 49, 98, 95, 89, 77, 53, 5, 10, 20, 40, 80, 59, 17, 34, 68,
            35, 70, 39, 78, 55, 9, 18, 36, 72, 43, 86, 71, 41, 82, 63, 25, 50,
            100, 99, 97, 93, 85, 69, 37, 74, 47, 94, 87, 73, 45, 90, 79, 57, 13,
            26, 52, 3, 6, 12, 24, 48, 96, 91, 81, 61, 21, 42, 84, 67, 33, 66,
            31, 62, 23, 46, 92, 83, 65, 29, 58, 15, 30, 60, 19, 38, 76, 51, 1],

        103 => vec![1, 5, 25, 22, 7, 35, 72, 51, 49, 39, 92, 48, 34, 67, 26, 27,
            32, 57, 79, 86, 18, 90, 38, 87, 23, 12, 60, 94, 58, 84, 8, 40, 97,
            73, 56, 74, 61, 99, 83, 3, 15, 75, 66, 21, 2, 10, 50, 44, 14, 70,
            41, 102, 98, 78, 81, 96, 68, 31, 52, 54, 64, 11, 55, 69, 36, 77, 76,
            71, 46, 24, 17, 85, 13, 65, 16, 80, 91, 43, 9, 45, 19, 95, 63, 6,
            30, 47, 29, 42, 4, 20, 100, 88, 28, 37, 82, 101, 93, 53, 59, 89, 33,
            62, 1],

        107 => vec![1, 2, 4, 8, 16, 32, 64, 21, 42, 84, 61, 15, 30, 60, 13, 26,
            52, 104, 101, 95, 83, 59, 11, 22, 44, 88, 69, 31, 62, 17, 34, 68,
            29, 58, 9, 18, 36, 72, 37, 74, 41, 82, 57, 7, 14, 28, 56, 5, 10, 20,
            40, 80, 53, 106, 105, 103, 99, 91, 75, 43, 86, 65, 23, 46, 92, 77,
            47, 94, 81, 55, 3, 6, 12, 24, 48, 96, 85, 63, 19, 38, 76, 45, 90,
            73, 39, 78, 49, 98, 89, 71, 35, 70, 33, 66, 25, 50, 100, 93, 79, 51,
            102, 97, 87, 67, 27, 54, 1],

        109 => vec![1, 6, 36, 107, 97, 37, 4, 24, 35, 101, 61, 39, 16, 96, 31,
            77, 26, 47, 64, 57, 15, 90, 104, 79, 38, 10, 60, 33, 89, 98, 43, 40,
            22, 23, 29, 65, 63, 51, 88, 92, 7, 42, 34, 95, 25, 41, 28, 59, 27,
            53, 100, 55, 3, 18, 108, 103, 73, 2, 12, 72, 105, 85, 74, 8, 48, 70,
            93, 13, 78, 32, 83, 62, 45, 52, 94, 19, 5, 30, 71, 99, 49, 76, 20,
            11, 66, 69, 87, 86, 80, 44, 46, 58, 21, 17, 102, 67, 75, 14, 84, 68,
            81, 50, 82, 56, 9, 54, 106, 91, 1],

        113 => vec![1, 3, 9, 27, 81, 17, 51, 40, 7, 21, 63, 76, 2, 6, 18, 54,
            49, 34, 102, 80, 14, 42, 13, 39, 4, 12, 36, 108, 98, 68, 91, 47, 28,
            84, 26, 78, 8, 24, 72, 103, 83, 23, 69, 94, 56, 55, 52, 43, 16, 48,
            31, 93, 53, 46, 25, 75, 112, 110, 104, 86, 32, 96, 62, 73, 106, 92,
            50, 37, 111, 107, 95, 59, 64, 79, 11, 33, 99, 71, 100, 74, 109, 101,
            77, 5, 15, 45, 22, 66, 85, 29, 87, 35, 105, 89, 41, 10, 30, 90, 44,
            19, 57, 58, 61, 70, 97, 65, 82, 20, 60, 67, 88, 38, 1],

        p => panic!("unknown modulus: {}", p)
    }
}

pub fn is_power_of_2<I>(x: I) -> bool
    where I: std::ops::Sub<Output=I> + std::ops::BitAnd<Output=I> +
             num::Zero + num::One + std::cmp::PartialEq + Clone
{
    (x.clone() & (x - I::one())) == I::zero()
}

// function that computes the number of carry digits you need to add n base-q digits
// together
pub fn num_carry_digits_to_add_n_digits(q: u16, n: usize) -> usize {
    ((n * (q as usize - 1)) as f64).log(q as f64).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn crt_conversion() {
        let mut rng = Rng::new();
        let ps = &PRIMES[..25];
        let modulus = product(ps);

        for _ in 0..128 {
            let x = rng.gen_u128() % modulus;
            assert_eq!(crt_inv(ps, &crt(ps, x)), x);
        }
    }

    #[test]
    fn factoring() {
        let mut rng = Rng::new();
        for _ in 0..16 {
            let mut ps = Vec::new();
            let mut q: u128 = 1;
            for &p in PRIMES.iter() {
                if rng.gen_bool() {
                    match q.checked_mul(p as u128) {
                        None => break,
                        Some(z) => q = z,
                    }
                    ps.push(p);
                }
            }
            assert_eq!(factor(q), ps);
        }
    }

    #[test]
    fn discrete_log() {
        let mut rng = Rng::new();
        for _ in 0..128 {
            let i = rng.gen_u16() as usize % NPRIMES;
            let q = PRIMES_SKIP_2[i];
            let tt = dlog_truth_table(q);
            let g = PRIMITIVE_ROOTS[i];
            let x = rng.gen_u16() % q;
            if x == 0 {
                continue;
            }
            let z = powm(g, tt[x as usize], q);
            assert_eq!(z, x);
            assert_eq!(z, exp_truth_table(q)[tt[x as usize] as usize]);
        }
    }

    #[test]
    fn bits() {
        let mut rng = Rng::new();
        for _ in 0..128 {
            let x = rng.gen_u128();
            assert_eq!(u128_from_bits(&to_bits(x, 128)), x);
        }
    }

    #[test]
    fn base_q_conversion() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_u16() % 111);
            let x = rng.gen_usable_u128(q);
            let y = as_base_q(x, q);
            let z = from_base_q(&y, q);
            assert_eq!(x, z);
        }
    }

    #[test]
    fn padded_base_q_conversion() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_u16() % 111);
            let x = rng.gen_usable_u128(q);
            let y = padded_base_q_128(x, q);
            let z = from_base_q(&y, q);
            assert_eq!(x, z);
        }
    }

    #[test]
    fn base_q_addition() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_u16() % 111);
            let n = digits_per_u128(q) - 2;
            println!("q={} n={}", q, n);
            let Q = (q as u128).pow(n as u32);

            let x = rng.gen_u128() % Q;
            let y = rng.gen_u128() % Q;

            let mut xp = padded_base_q(x,q,n);
            let yp = as_base_q(y,q);

            let zp = base_q_add(&xp, &yp, q);

            let z = from_base_q(&zp, q);

            assert_eq!((x+y) % Q, z);
        }
    }


    #[test]
    fn max_carry_digits() {
        let mut rng = Rng::new();
        for _ in 0..1000 {
            let q = 2 + (rng.gen_u16() % 254);
            let n = 2 + (rng.gen_usize() % 1000);
            let xs = vec![BigInt::from(q-1); n];
            let p: BigInt = xs.iter().sum();
            let (_, ds) = p.to_radix_le(q as u32);
            assert_eq!(ds.len(), num_carry_digits_to_add_n_digits(q,n));
        }
    }
}
