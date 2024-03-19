# trsltx
Tools for automatic translation of texts written with LaTeX

Usage: go in the `trsltx` directory and run

```bash
cargo run
```

By default the French LaTeX file `test/simple_fr.tex` is translated into english in `test/simple_en.tex`

For changing the default behavior do, for instance

```bash
cargo run -- -i test/simple_fr.tex -o test/simple_de.tex
```

Or

```bash
cargo build --release
./target/release/trsltx -i test/simple_fr.tex -o test/simple_de.tex
```
