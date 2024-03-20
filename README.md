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

The translation is completed using a Large Language Model (LLM) and may contain some LaTeX errors.
Therefore, it is essential to review and manually correct the translated code as necessary.

The original LaTeX file has to be split in not too long chunks by using markers
`%trsltx-split` in the .tex file on  single lines.

It is also possible to mark a region that should not be translated with the markers
`%trsltx-begin-ignore` and `%trsltx-end-ignore` on single lines. Ignored regions should not contain
`%trsltx-split` markers. See the file `test/simple_fr.tex` for an example.

