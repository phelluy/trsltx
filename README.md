# trsltx
Tools for automatic translation of texts written with LaTeX.

You need first to get a valid API key from [https://textsynth.com/](https://textsynth.com/)
and put it in a file named `api_key.txt` in the working directory or in an environment variable by

```bash
export TEXTSYNTH_API_KEY=<the_api_key>
```

Usage: go in the `trsltx` directory and run

```bash
cargo run
```

By default the French LaTeX file `test/simple_fr.tex` is translated into english in `test/simple_en.tex`.

The languages are specified in the filename by the `_xy` mark, where `xy` is the abbreviated language name.
Currently, the available languages are: `en`, `fr`, `es`, `de`, `it`, `pt`, `ru`. 

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

The translation is done with a Large Language Model (LLM). It is possible that some LateX errors occur in the translation by the LLM. You have to correct them by hand.
