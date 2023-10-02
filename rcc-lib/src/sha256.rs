use rcc::traits::UInt32;
use rcc::{component_of, WithGlobalBuilder, Builder};

const H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
];

const K: [u32; 64] = [
   0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
   0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
   0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
   0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
   0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
   0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
   0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
   0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
];

#[component_of(T::global_builder())]
fn ch<T: UInt32 + WithGlobalBuilder>(x: T, y: T, z: T) -> T {
    (x & y) ^ (!x & z)
}

#[component_of(T::global_builder())]
fn maj<T: UInt32 + WithGlobalBuilder>(x: T, y: T, z: T) -> T {
    (x & y) ^ (x & z) ^ (y & z)
}

#[component_of(T::global_builder())]
fn bsigma0<T: UInt32 + WithGlobalBuilder>(x: T) -> T {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[component_of(T::global_builder())]
fn bsigma1<T: UInt32 + WithGlobalBuilder>(x: T) -> T {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[component_of(T::global_builder())]
fn ssigma0<T: UInt32 + WithGlobalBuilder>(x: T) -> T {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[component_of(T::global_builder())]
fn ssigma1<T: UInt32 + WithGlobalBuilder>(x: T) -> T {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[component_of(T::global_builder())]
pub fn sha256_compression<T: UInt32 + WithGlobalBuilder>(input: [T; 16], hin: [T; 8]) -> [T; 8] {
    let mut w: Vec<T> = Vec::with_capacity(64);

    let mut a = hin[0];
    let mut b = hin[1];
    let mut c = hin[2];
    let mut d = hin[3];
    let mut e = hin[4];
    let mut f = hin[5];
    let mut g = hin[6];
    let mut h = hin[7];

    let k_vec: Vec<T> = K.iter().map(|u| T::from_const(*u)).collect();
    let kk: [T; 64] = k_vec.try_into().unwrap_or_else(|_| unimplemented!());

    for t in 0..64 {
        let __for_loop = T::global_builder().new_context("for_loop".into());
        if t < 16 {
            w.push(input[t])
        } else {
            let __a = T::global_builder().new_context("a".into());
                w.push(ssigma1(w[t-2]) + w[t-7] + ssigma0(w[t-15]) + w[t-16])
        };

        let mut t1 = w[t] + kk[t];

        let __b = T::global_builder().new_context("b".into());
        t1 = t1 + h + bsigma1(e) + ch(e,f,g);
        let t2 = bsigma0(a) + maj(a,b,c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    [
        hin[0] + a,
        hin[1] + b,
        hin[2] + c,
        hin[3] + d,
        hin[4] + e,
        hin[5] + f,
        hin[6] + g,
        hin[7] + h,
    ]
}

#[component_of(T::global_builder())]
pub fn sha256<T: UInt32 + WithGlobalBuilder>(input: Vec<T>) -> [T; 8] {
    let n_bits = input.len() * 32;

    let num_zeros = 512 - ((n_bits + 64) % 512) - 1;

    let length_hi = ((n_bits as u64) >> 32) as u32;
    let length_lo = n_bits as u32;

    // println!("{length_hi}, {length_lo}"); println!("num_zeros: {num_zeros}");

    let mut padded = input.clone();

    if num_zeros >= 31 {
        padded.push(UInt32::from_const(1u32 << 31));
        (0..num_zeros / 32).for_each(|_| {
            padded.push(UInt32::from_const(0));
        });
    } else {
        todo!()
    }

    padded.push(UInt32::from_const(length_hi));
    padded.push(UInt32::from_const(length_lo));

    // println!("length: {}", padded.len());

    let blocks = padded.chunks(16);

    let mut state = [
        UInt32::from_const(H[0]),
        UInt32::from_const(H[1]),
        UInt32::from_const(H[2]),
        UInt32::from_const(H[3]),
        UInt32::from_const(H[4]),
        UInt32::from_const(H[5]),
        UInt32::from_const(H[6]),
        UInt32::from_const(H[7]),
    ];

    for block in blocks {
        state = sha256_compression(block.try_into().unwrap(), state)
    }

    state
}
