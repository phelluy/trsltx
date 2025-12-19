# trsltx
Tools for automatic translation of texts written with LaTeX.

You need first to run a [llama.cpp](https://github.com/ggerganov/llama.cpp) server in order to access a Large Language Model (LLM). 
See the [installation instructions](https://github.com/ggerganov/llama.cpp?tab=readme-ov-file#usage).

```bash
<path_to_llama_cpp_directory>/server -m <your_model.gguf> -c 32768
```

We recommend using the **Mistral Small 3 24B** model (`mistral-small-3.2-24b-instruct-2409.Q4_K_M.gguf` or similar quantization), which gives excellent results for LaTeX translation. Smaller models should also work fine.

`trsltx` will connect to `http://localhost:8080/completion` by default.

Alternatively, you can use the [TextSynth](https://textsynth.com/) API.
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
Currently, the available languages are: `en`, `fr`, `es`, `de`, `it`, `pt`, `ru`, `nl`.

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

`cargo install` is the recommended method: it takes into account bug fixes both in the parser `ltxprs` and in the translator `trsltx`.

How it works: the input file is split into chunks of text using a smart splitting algorithm (based on semantic structure). The chunks are automatically translated with a LLM hosted on the local `llama.cpp` server.
The `trsltx` tool uses a unique feature of the `llama.cpp` API, which allows the possibility to use a formal grammar to constrain the generated output. 
See the [grammar options of `llama.cpp`](https://github.com/ggerganov/llama.cpp/blob/master/grammars/README.md) (GBNF grammars).

Anyway, because the translation is completed using a LLM, it may still contain some LaTeX errors.
It is recommended to review and manually correct the translated code as necessary.

By default, `trsltx` uses a smart splitting algorithm (based on semantic structure) for creating chunks of text and proceeds directly to translation of these chunks.

```bash
cargo run -- -i fr -o en -f test/simple.tex
```

If the chunking is not satisfactory, manual splitting remains possible by using the `--manual` option.
The original LaTeX file is split in not too long chunks by using markers
`%trsltx-split` in the .tex file on single lines. `trsltx` will complain if a chunk
is too long. It is possible to specify a split length with the `-l` option of `trsltx`.
In the process an intermediate file `test/simple_fr.tex` is generated with split markers.
It is possible to adjust the position of the markers manually.

```bash
cargo run -- --manual -i fr -o en -f test/simple.tex
```

Each chunk is analyzed using a lightweight parser for a subset of the LaTeX syntax (see [ltxprs](https://github.com/phelluy/ltxprs)). A special grammar is generated for each fragment, which encourages the LLM to stick to the original text. This discourages invented labels, references or citations. In addition, LaTeX commands that are not in the original text are less likely to be generated.

The grammar function is deactivated if the light syntax analyzer fails. The chunk is partially translated if the server returns an error. In this case, the translation must be corrected manually...

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
* The parser has other limitations (such as `\verbatim` envs). See [ltxprs](https://github.com/phelluy/ltxprs) for limitations and possible workarounds.
 
**Note:** We pay tribute to the pioneering spirit of **Fabrice Bellard**, creator of TextSynth (and QEMU, FFmpeg, etc.), whose early work on constrained inference inspired this tool.

