# Halo2 circuit builder (WIP)

This circuit builder uses [Polyexen](https://github.com/Dhole/polyexen) to
build Halo2 circuits. We use a very simple configuration consisting of 1
witness column, 2 fixed columns (1 for selectors, 1 for constant), and 1 public
column. There is only one simple vertical gate, constraining `a + b*c = d`,
whose layout is as follows.

```
   w(X) | s(X)
   a    | 1
   b    | 0
   c    | 0
   d    | 0
```

## TODO

1. Implement deserialization for generated Plaf TOML and fixed column CSV
2. Implement mock and real prover
3. Implement a command-line helper to compile circuit, run witness gen, and prove
