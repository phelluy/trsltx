# trsltx
Tools for automatic translation of texts written with LaTeX.

You need first to get a valid API key from [https://textsynth.com/](https://textsynth.com/)
and put it in a file named `api_key.txt` in the working directory or in an environment variable by

```bash
export TEXTSYNTH_API_KEY=<the_api_key>
```

Usage: go in the `trsltx` directory and run (you need a working install of [Rust](https://www.rust-lang.org/tools/install))

```bash
cargo run
```

By default, the French LaTeX file `test/simple.tex` is translated into English in `test/simple_en.tex`.

The languages are specified in the filename by the `_xy` mark, where `xy` is the abbreviated language name.
Currently, the available languages are: `en`, `fr`, `es`, `de`, `it`, `pt`, `ru`. 

For changing the default behavior do, for instance

```bash
cargo run -- -i fr -o de -f test/simple.tex
```

Or, for installing `trsltx` in your user account

```bash
cargo install --path .
trsltx --help
trsltx -i fr -o de -f test/simple.tex
```

`cargo install`is the recommend method: it takes into accound bug fixes both in the parser `ltxprs`and in the translator `trsltx`.

The translation is completed using a Large Language Model (LLM) available on the Texsynth server. It may contain some LaTeX errors.
Therefore, it is essential to review and manually correct the translated code as necessary.

`trsltx` uses a unique feature of the Textsynth API, which allows the possibility to use a formal BNF grammar to constraint the generated output. 
See [https://textsynth.com/documentation.html#grammar](https://textsynth.com/documentation.html#grammar).

The original LaTeX file is split in not too long chunks by using markers
`%trsltx-split` in the .tex file on single lines. `trsltx` will complain if a chunk
is too long. It is possible to specify a split length with the `-l` option of `trsltx`.
In the process an intermediate file `test/simple_fr.tex` is generated with split markers.
For now, the automatic split is not very powerful. It is recomended to adjust the position of the
markers manually if the translation is not satisfactory.

Each chunk is analyzed using a lightweight parser for a subset of the LaTeX syntax (see [ltxprs](https://github.com/phelluy/ltxprs)). A special grammar is generated for each fragment, which encourages the LLM to stick to the original text. This discourages invented labels, references or citations. In addition, LaTeX commands that are not in the original text are less likely to be generated.

The grammar function is deactivated if the light syntax analyser fails. The chunk is partially translated if the server returns an error. In this case, the translation must be corrected manually...

It is also possible to mark a region that should not be translated with the markers
`%trsltx-begin-ignore` and `%trsltx-end-ignore` on single lines. Ignored regions should not contain
`%trsltx-split` markers. See the file `test/simple.tex` for an example.

Here are a few tips for improved results:

* Your initial .tex file must compile without any error, of course. Be careful, the LaTeX compiler sometimes ignores unpaired braces `{...}`, which `trsltx` will not accept.
* If a part of your initial .tex file is not recognized by the parser, comment it, remove the temporary file and restart `trsltx`.
* You can define fancy LaTeX macros, but only in the preamble, before `\begin{document}`.
* Give meaningful names to your macros for helping the translator (e.g. don't call a macro that displays the energy `\foo`. A better choice is `\energy`!).
* Don't use alternatives to the following commands: `\cite`, `\label`, `\ref`. Otherwise, the labels, refs and citations may be lost in translation.
* Avoid using `%trsltx-split` in the middle of math formulas, `{...}` groups or `\begin ... \end` environments. 
* The parser `ltxprs`does not accept special characters (like `$` or `%`) in file names or url strings. It will fail even if the line is commented. A workaround is to define an alias without special character in the preamble.



