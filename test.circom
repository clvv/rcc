pragma circom 2.1.2;

template Sub(M) {
    signal input a;
    signal input b;

    signal c[M+1];

    signal output d;

    c[0] <== a * b;

    for (var i = 0; i < M; i++) {
        c[i+1] <== c[i] ** 2;
    }

    d <== c[M];
}

template Gen(N) {
    signal input val;
    signal output a[N];
    signal output b[N];

    for (var i = 0; i < N; i++) {
        a[i] <== val + i;
        b[i] <== val - i;
    }
}

template Main(N, M) {
    signal input val;
    signal a[N];
    signal b[N];

    (a, b) <== Gen(N)(val);

    signal c[N];
    signal output d;

    for (var i = 0; i < N; i++) {
        c[i] <== Sub(M)(a[i], b[i]);
    }

    var sum;
    for (var i = 0; i < N; i++) {
        sum += c[i];
    }

    d <== sum;
    log(d);
}

component main = Main(100, 100);
