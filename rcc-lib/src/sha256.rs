use rcc::traits::UInt as UInt32;

const H0: u32 = 0x6a09e667;
const H1: u32 = 0xbb67ae85;
const H2: u32 = 0x3c6ef372;
const H3: u32 = 0xa54ff53a;
const H4: u32 = 0x510e527f;
const H5: u32 = 0x9b05688c;
const H6: u32 = 0x1f83d9ab;
const H7: u32 = 0x5be0cd19;

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

fn ch<T: UInt32>(x: T, y: T, z: T) -> T {
    (x & y) ^ (!x & z)
}

fn maj<T: UInt32>(x: T, y: T, z: T) -> T {
    (x & y) ^ (x & z) ^ (y & z)
}

fn bsigma0<T: UInt32>(x: T) -> T {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

fn bsigma1<T: UInt32>(x: T) -> T {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

fn ssigma0<T: UInt32>(x: T) -> T {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}


fn ssigma1<T: UInt32>(x: T) -> T {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}


#[allow(non_snake_case)]
pub fn sha256_compression<T: UInt32>(input: [T; 16], H: [T; 8]) -> [T; 8] {
    let mut w: Vec<T> = Vec::with_capacity(64);

    let mut a = H[0];
    let mut b = H[1];
    let mut c = H[2];
    let mut d = H[3];
    let mut e = H[4];
    let mut f = H[5];
    let mut g = H[6];
    let mut h = H[7];

    let mut T1;
    let mut T2;

    for t in 0..64 {
        w[t] = if t < 16 {
            input[t]
        } else {
            ssigma1(w[t-2]) + w[t-7] + ssigma0(w[t-15]) + w[t-16]
        };

        T1 = h + bsigma1(e) + ch(e,f,g) + K[t] + w[t];
        T2 = bsigma0(a) + maj(a,b,c);

        h = g;
        g = f;
        f = e;
        e = d + T1;
        d = c;
        c = b;
        b = a;
        a = T1 + T2;
    }

    [
        H[0] + a,
        H[1] + b,
        H[2] + c,
        H[3] + d,
        H[4] + e,
        H[5] + f,
        H[6] + g,
        H[7] + h,
    ]
}

pub fn sha256<T: UInt32>(input: Vec<T>) -> [T; 8] {
    let n_bits = input.len() * 32;
    let n_blocks = (n_bits + 64) / 512 + 1;

    let num_zeros = 512 - (n_bits % 512) - 1;

    todo!()
}
