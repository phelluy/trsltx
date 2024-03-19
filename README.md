# trsltx
Tools for automatic translation of texts written with LaTeX

You need first to get a valid API key from [https://textsynth.com/](https://textsynth.com/)
and put it in a file named `api_key.txt` in the working directory or in an environment variable by

```bash
export TEXTSYNTH_API_KEY=<the_api_key>
```

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
cp ./target/release/trsltx .
./trsltx -i test/simple_fr.tex -o test/simple_de.tex
```
