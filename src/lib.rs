//! # trsltx
//! Tools for automatic translation of texts written with LaTeX.
//!
//! You need first to get a valid API key from [https://textsynth.com/](https://textsynth.com/)
//! and put it in a file named `api_key.txt` in the working directory or in an environment variable by
//!
//! ```bash
//! export TEXTSYNTH_API_KEY=<the_api_key>
//! ```
//!
//! Usage: go in the `trsltx` directory and run
//!
//! ```bash
//! cargo run
//! ```
//!
//! By default, the French LaTeX file `test/simple.tex` is translated into English in `test/simple_en.tex`.
//!
//! The languages are specified in the filename by the `_xy` mark, where `xy` is the abbreviated language
//!  name.
//! Currently, the available languages are: `en`, `fr`, `es`, `de`, `it`, `pt`, `ru`.
//!
//! For changing the default behavior do, for instance
//!
//! ```bash
//! cargo run -- -i fr -o de -f test/simple.tex
//! ```
//!
//! Or
//!
//! ```bash
//! cargo install --path .
//! trsltx --help
//! trsltx -i fr -o de -f test/simple.tex
//! ```
//!
//! The translation is completed using a Large Language Model (LLM) available on the Texsynth server.
//! It may contain some LaTeX errors.
//! Therefore, it is essential to review and manually correct the translated code as necessary.
//!
//! `trsltx` uses a unique feature of the Textsynth API, which allows the possibility to use a formal
//!  BNF grammar to constraint the generated output.
//! See [https://textsynth.com/documentation.html#grammar](https://textsynth.com/documentation.html#grammar).
//!
//! The original LaTeX file has to be split in not too long chunks by using markers
//! `%trsltx-split` in the .tex file on single lines. `trsltx` will complain if a chunk
//! is too long. It is possible to specify a split length with the `-l` option of `trsltx`.
//!  In the process an intermediate file `test/simple_fr.tex` is generated with split markers.
//! The automatic split is not very powerful. It is recomended to adjust the position of the
//! markers manually if the translation is not satisfactory.
//!
//! Each chunk is analyzed using a lightweight parser for a subset of the LaTeX syntax.
//! A special grammar is generated for each fragment, which encourages the LLM to stick to the original text.
//! This discourages invented labels, references or citations.
//! In addition, LaTeX commands that are not in the original text are less likely to be generated.
//!
//! The grammar feature is deactivated if the light parser fails.
//! The chunk is partially translated if the server returns an error.
//!
//! It is also possible to mark a region that should not be translated with the markers
//! `%trsltx-begin-ignore` and `%trsltx-end-ignore` on single lines. Ignored regions should not contain
//! `%trsltx-split` markers. See the file `test/simple.tex` for an example.
//!
//! Here are a few tips for improved results:
//!
//! * Your initial .tex file must compile without any error, of course.
//! Be careful, the LaTeX compiler sometimes ignores unpaired braces `{...}`, which `trsltx` will not accept.
//! * You can define fancy LaTeX macros, but only in the preamble, before `\begin{document}`.
//! * Give meaningful names to your macros for helping the translator
//! (e.g. don't call a macro that displays the energy `\foo`. A better choice is `\energy`!).
//! * Don't use alternatives to the following commands: `\cite`, `\label`, `\ref`.
//! Otherwise, the labels, refs and citations may be lost in translation.
//! * Avoid using `%trsltx-split` in the middle of math formulas,
//!  `{...}` groups or `\begin ... \end` environments.

use std::io::Write;

use ltxprs::LtxNode;

#[derive(Debug, Clone)]
enum ChunkType {
    Translate,
    Unchanged,
}

#[derive(Debug)]
pub struct Trsltx {
    input_lang: String,
    output_lang: String,
    input_file_name: String,
    output_file_name: String,
    model_name: String,
    preamble: String,
    body: String,
    afterword: String,
    body_translated: String,
    chunks: Vec<(String, ChunkType)>,
}

impl Trsltx {
    pub fn new(
        input_lang: &str,
        output_lang: &str,
        input_file_name: &str,
        output_file_name: &str,
        model_name: &str,
    ) -> Trsltx {
        Trsltx {
            input_lang: input_lang.to_string(),
            output_lang: output_lang.to_string(),
            input_file_name: input_file_name.to_string(),
            output_file_name: output_file_name.to_string(),
            model_name: model_name.to_string(),
            preamble: String::new(),
            body: String::new(),
            afterword: String::new(),
            body_translated: String::new(),
            chunks: Vec::new(),
        }
    }
    pub fn read_file(&mut self) -> Result<(), String> {
        let input_file = std::fs::read_to_string(&self.input_file_name)
            .map_err(|e| format!("Cannot read file: {:?}", e))?;
        // replace \r characters by nothing (appear in Windows files...)
        let input_file = input_file.replace('\r', "");
        //let input_file = input_file.replace("\\end{document}", "\\commandevide\n\\end{document}");
        //let input_file = input_file.replace("\\end{document}", "\\commandevide\n\\end{document}");

        let mut input_file = input_file.split("\\begin{document}");
        self.preamble = input_file
            .next()
            .ok_or("Error in extracting preamble.")?
            .to_string();
        let mut input_file = input_file
            .next()
            .ok_or("No \\begin{document} in the tex file.")?
            .split("\\end{document}");
        self.body = input_file
            .next()
            .ok_or("Error in extracting body.")?
            .to_string();
        self.afterword = input_file
            .next()
            .ok_or("No \\end{document} in the tex file.")?
            .to_string();
        Ok(())
    }

    /// Translate the body of the file
    pub fn translate(&mut self) {
        let preamble = adjust_preamble_lang(
            self.preamble.clone(),
            self.input_lang.as_str(),
            self.output_lang.as_str(),
        );
        match preamble {
            Ok(preamble) => self.preamble = preamble,
            Err(e) => println!("Found no babel option in preamble: {:?}", e),
        }
        self.translate_chunks();
    }

    /// pass the body to print_split a generate a latex string with
    /// the "%trsltx-split" markers
    pub fn generate_split_latex(&self, split_length: usize) -> String {
        let body = self.body.clone();
        let ltxparse = LtxNode::new(body.as_str());
        let body = ltxparse.print_split(0, String::new(), split_length);
        //trim body
        let body = body.trim();
        //remove heading { and trailing }
        let len = body.len();
        let body = if len >= 2 {
            body[1..len - 1].to_string()
        } else {
            body.to_string()
        };

        let latex = self.preamble.clone()
            + "\\begin{document}\n"
            + &body
            + "\n\\end{document}\n"
            + &self.afterword.clone();

        println!("code: {}", latex);

        latex
    }

    /// Extract the chunks to be translated from the body
    /// the chunks are separated by the string "%trsltx-split\n"
    /// or are enclosed between "%trsltx-begin-ignore\n" and "%trsltx-end-ignore\n"
    /// by defaults, the chunks are marked as Translate
    /// the chunks enclosed between "%trsltx-begin-ignore\n" and "%trsltx-end-ignore\n"
    /// are marked as Unchanged
    pub fn extract_chunks(&mut self) -> Result<(), String> {
        let toscan = self.body.clone();
        // add %trsltx-split before each %trsltx-begin-ignore
        let toscan = toscan.replace(
            "%trsltx-begin-ignore",
            "%trsltx-split\n%trsltx-begin-ignore",
        );
        // add %trsltx-split after each %trsltx-end-ignore
        let toscan = toscan.replace("%trsltx-end-ignore", "%trsltx-end-ignore\n%trsltx-split\n");
        // split the body into chunks
        let chunks = toscan.split("%trsltx-split\n");
        for chunk in chunks {
            let cchunk = chunk.trim().replace("%trsltx-split\n", "");
            if cchunk.contains("%trsltx-begin-ignore") {
                if !cchunk.contains("%trsltx-end-ignore") {
                    return Err("Unbalanced %trsltx-begin-ignore".to_string());
                }
                self.chunks.push((cchunk.to_string(), ChunkType::Unchanged));
            } else if cchunk.contains("%trsltx-end-ignore") {
                return Err("Unbalanced %trsltx-end-ignore".to_string());
            } else {
                self.chunks.push((cchunk.to_string(), ChunkType::Translate));
            }
        }

        let numchunks = self.chunks.len();

        for i in 0..numchunks {
            let (s, t) = self.chunks[i].clone();
            if let ChunkType::Unchanged = t {
                // s = s.replace("%trsltx-begin-ignore\n", "");
                // s = s.replace("%trsltx-end-ignore\n", "");
                self.chunks[i] = (s, ChunkType::Unchanged);
            }
        }
        // mark last chunk as Unchanged
        // if numchunks > 0 {
        //     let (s, _) = self.chunks[numchunks - 1].clone();
        //     self.chunks[numchunks - 1] = (s, ChunkType::Unchanged);
        // }
        println!("{:?}", self.chunks);
        Ok(())
    }

    /// Same as function "translate"
    // this function should not fail because if it encounters an error
    // it translates the chunk without the grammar analysis or
    // on the worst errors, it leaves the chunk unchanged
    pub fn translate_chunks(&mut self) {
        let mut body_translated = String::new();
        let numchunks = self.chunks.len();
        let mut count = 0;
        for (chunk, t) in self.chunks.iter() {
            println!("------------------------------------------");
            match t {
                ChunkType::Translate => {
                    count += 1;
                    let chunk_length = chunk.len();
                    let max_chunk_length = 4000;
                    let trs_try = if chunk_length >= max_chunk_length {
                        println!("{:?}", chunk);
                        println!(
                            "Chunk too long: {} above {}",
                            chunk_length, max_chunk_length
                        );
                        println!("Leave chunk {} of {} unchanged", count, numchunks);
                        Ok(chunk.to_string())
                    } else {
                        println!("Translating chunk {} of {}", count, numchunks);
                        translate_one_chunk(
                            chunk.as_str(),
                            self.input_lang.as_str(),
                            self.output_lang.as_str(),
                            self.model_name.clone(),
                        )
                    };
                    match trs_try {
                        Ok(trs_chunk) => {
                            // append the split message
                            // so that the translated file
                            // can be reused by trsltx
                            if count > 1 {
                                body_translated.push_str("\n%trsltx-split\n");
                            }
                            body_translated.push_str(trs_chunk.as_str());
                        }
                        Err(e) => {
                            println!("Error in translating chunk: {:?}", e);
                            println!("Leave chunk {} of {} unchanged", count, numchunks);
                            body_translated.push_str(chunk.as_str());
                        }
                    }
                }
                ChunkType::Unchanged => {
                    count += 1;
                    println!("    Copying chunk {} of {}", count, numchunks);
                    body_translated.push_str(chunk.as_str());
                }
            }
        }
        // last cleaning:
        // remove the %trsltx-split immediately following %trsltx-end-ignore
        body_translated = body_translated.replace(
            "%trsltx-end-ignore\n%trsltx-split\n",
            "%trsltx-end-ignore\n",
        );
        self.body_translated = body_translated;
    }

    pub fn write_file(&self) -> Result<(), String> {
        let mut output_file = std::fs::File::create(&self.output_file_name)
            .map_err(|e| format!("Cannot create file: {:?}", e))?;
        output_file
            .write_all(self.preamble.as_bytes())
            .map_err(|e| format!("Cannot write to file: {:?}", e))?;

        // write the translated body
        // create the latex env trsltx  in case the translatex chunk is enclosed between
        // \begin{trsltx} and \end{trsltx}
        output_file
            .write_all(
                "\\newenvironment{trsltx}{}{}\n\n\\newcommand{\\commandevide}{}\n\\begin{document}"
                    .as_bytes(),
            )
            .map_err(|e| format!("Cannot write to file: {:?}", e))?;

        output_file
            .write_all(self.body_translated.as_bytes())
            .map_err(|e| format!("Cannot write to file: {:?}", e))?;
        output_file
            .write_all("\\end{document}".as_bytes())
            .map_err(|e| format!("Cannot write to file: {:?}", e))?;
        output_file
            .write_all(self.afterword.as_bytes())
            .map_err(|e| format!("Cannot write to file: {:?}", e))?;

        Ok(())
    }
}

/// If the babel latex option is detected, replace the source
/// language in the babel option by the target language
pub fn adjust_preamble_lang(
    preamble: String,
    inlang: &str,
    outlang: &str,
) -> Result<String, String> {
    let target_lang = get_lang_name(outlang)?.to_lowercase();
    let source_lang = get_lang_name(inlang)?.to_lowercase();
    let mut preamble = preamble.replace(source_lang.as_str(), target_lang.as_str());
    if target_lang == "russian" {
        // if \usepackage[T1]{fontenc} is not present in the preamble
        // issue a warning
        if !preamble.contains("\\usepackage[T1]{fontenc}") {
            println!(r#"Warning: \\usepackage[T1]{{fontenc}} is not present in the preamble"#);
            println!(r#"The Russian language requires \\usepackage[T2A]{{fontenc}}"#);
            println!(r#"Add \\usepackage[T2A]{{fontenc}} to the preamble"#);
        }
        preamble = preamble.replace(
            r#"\usepackage[T1]{fontenc}"#,
            r#"\usepackage[T2A]{fontenc}"#,
        );
    }
    Ok(preamble)
}

/// Get the long language name from the short two-letter one
pub fn get_lang_name(lang: &str) -> Result<String, String> {
    // list of known languages
    const LANGUAGES: [(&str, &str); 7] = [
        ("en", "English"),
        ("fr", "French"),
        ("es", "Spanish"),
        ("de", "German"),
        ("it", "Italian"),
        ("pt", "Portuguese"),
        ("ru", "Russian"),
    ];

    // build a dictionnary from the list of languages
    let mut lang_dict = std::collections::HashMap::new();
    for (k, v) in LANGUAGES.iter() {
        lang_dict.insert(k.to_string(), v.to_string());
    }

    let lang = lang_dict.get(lang).ok_or(
        "The supported languages are: en,fr,es,de,it,pt,ru. Unsupported language: ".to_owned()
            + lang,
    )?;
    Ok(lang.to_string())
}

/// one chat operation with the textsynth LLM
/// send the question
/// and returns an answer
#[allow(dead_code)]
fn chat_with_ts(question: &str) -> Result<String, String> {
    // get the api key from the file "api_key.txt" or if the file does not exist, from the environment variable "TEXTSYNTH_API_KEY"
    let api_key = match std::fs::read_to_string("api_key.txt") {
        // if the file exists, get the api key from the file
        // removing the spaces and newlines with trim()
        Ok(api_key) => api_key.trim().to_string(),
        Err(_) => std::env::var("TEXTSYNTH_API_KEY").expect("You have to provide an api key in the file api_key.txt or by export TEXTSYNTH_API_KEY=api_key"),
    };

    // call the textsynth REST API
    //let url = "https://api.textsynth.com/v1/engines/mixtral_47B_instruct/chat";
    // also works well with the engine mistral_7B_instruct
    let url = "https://api.textsynth.com/v1/engines/mistral_7B_instruct/chat";
    let max_tokens = 2000;

    use serde_json::json;
    use serde_json::Value;

    let req = json!({
        "messages": [question],
        "temperature": 0.5,
        "max_tokens": max_tokens
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .expect("Failed to send request")
        .json::<Value>();

    let answer: String = match res {
        Ok(resp) => {
            let text = resp["text"]
                .as_str()
                .ok_or("The result of Textsynth does not contain text")?;
            //println!("{:?}", text);
            text.to_string()
        }
        Err(e) => {
            println!("Request error: {:?}", e);
            "".to_string()
        }
    };
    Ok(answer)
}

/// one completion operation with the textsynth LLM
/// send the question and a formal grammar (as Some(String) or None)
/// and returns an answer
fn complete_with_ts(
    prompt: &str,
    grammar: &Option<String>,
    model: String,
) -> Result<String, String> {
    // get the api key from the file "api_key.txt"
    //or if the file does not exist, from the environment variable "TEXTSYNTH_API_KEY"
    let api_key = match std::fs::read_to_string("api_key.txt") {
        // if the file exists, get the api key from the file
        // removing the spaces and newlines with trim()
        Ok(api_key) => api_key.trim().to_string(),
        Err(_) => std::env::var("TEXTSYNTH_API_KEY").map_err(|e| format!("You have to provide an api key in the file api_key.txt or by export TEXTSYNTH_API_KEY=api_key. Error: {:?}", e))?,
    };

    // call the textsynth REST API
    let url = match model.as_str() {
        "mistral47b" => "https://api.textsynth.com/v1/engines/mixtral_47B_instruct/completions",
        _ => "https://api.textsynth.com/v1/engines/mistral_7B_instruct/completions",
    };

    let max_tokens = 2000;

    use serde_json::json;
    use serde_json::Value;

    let req = match grammar {
        Some(gr) => {
            json!({
                "prompt": prompt,
                "temperature": 0.5,
                "max_tokens": max_tokens,
                "grammar": gr
            })
        }
        None => {
            // println!("No grammar");
            json!({
                "prompt": prompt,
                "temperature": 0.5,
                "max_tokens": max_tokens
            })
        }
    };
    //println!("Req= {:?}", req);
    println!("Translate with {}", model);
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .map_err(|e| format!("Failed to send request: {:?}", e))?
        .json::<Value>();
    println!("{:?}", res);

    let answer: String = match res {
        Ok(resp) => {
            //println!("{:?}", resp);
            let text = resp["text"]
                .as_str()
                .ok_or("The result of Textsynth does not contain text")?;
            //println!("{:?}", text);
            text.to_string()
        }
        Err(e) => {
            println!("Request error: {:?}", e);
            "".to_string()
        }
    };

    Ok(answer)
}

const PREPROMPT: &str = r#"
Q: Translate the following <lang_in> scientific text, formatted with LateX, into <lang_out>.
Keep the LateX syntax and formulas. The results must compile without errors with pdflatex.
Give only the result without preliminaries. 
Enclose the resulting LateX source between \begin{trsltx} and \end{trsltx}
Here is the <lang_in> LateX source:

"#;

/// translate a latex chunk using the textsynth LLM api
/// the preprompt is in the file "prompt.txt"
/// the api key is in the file "api_key.txt" or
/// in the environment variable "TEXTSYNTH_API_KEY"
fn translate_one_chunk(chunk: &str, input_lang: &str, output_lang: &str, model: String) -> Result<String, String> {
    println!("Translating chunk: {:?}", chunk);
    if chunk.trim() == r#"\commandevide"# || chunk.trim() == "" {
        println!("Empty chunk");
        // create a string containing \commandvide followed by a newline
        let s = "\\commandevide\n".to_string();
        return Ok(s);
    }
    // get the preprompt from a file
    // let mut prompt = std::fs::read_to_string("src/prompt.txt")
    //     .map_err(|_| "cannot read preprompt".to_string())?;
    // or directly from the const PREPROMPT
    let mut prompt = PREPROMPT.to_string();

    let input_lang = get_lang_name(input_lang)?.to_string();
    let output_lang = get_lang_name(output_lang)?.to_string();

    // in the prompt, replace <lang_in> by the input language and <lang_out> by the output language
    prompt = prompt.replace("<lang_in>", input_lang.as_str());
    prompt = prompt.replace("<lang_out>", output_lang.as_str());

    let question = format!("{}\n{}\nA:\n", prompt, chunk);
    // exit(0);
    //let trs_chunk = chat_with_ts(question.as_str());
    let ast_chunk = LtxNode::new(chunk);
    //let cmds = ast_chunk.extracts_commands();
    //println!("{:?}", ast_chunk);
    let grammar = match ast_chunk {
        LtxNode::Problem(_) => None,
        _ => Some(ast_chunk.to_ebnf().trim().to_string()),
    };
    //ast_chunk.print();
    println!("Grammar: {}", ast_chunk.to_ebnf());
    let mut distmin = std::usize::MAX;
    let mut iter = 0;
    let mut trs_chunk = "".to_string();
    let itermax = 4;
    // at most four attempts to get a translation
    while distmin > 1 && iter < itermax {
        // last iter without grammar
        let trs_try = if iter > itermax - 2 {
            complete_with_ts(question.as_str(), &None, model.clone())
        } else {
            complete_with_ts(question.as_str(), &grammar, model.clone())
        };
        let trs_try = match trs_try {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        let trs_try = trs_try.split("\\begin{trsltx}").collect::<Vec<&str>>();
        let trs_try = if trs_try.len() >= 2 {
            let trs_try = trs_try[1].split("\\end{trsltx}").collect::<Vec<&str>>()[0];
            trs_try.to_string()
        } else {
            "".to_string()
        };
        let trs_ltxnode = LtxNode::new(trs_try.as_str());
        let dist = ast_chunk.distance(&trs_ltxnode);
        println!("Syntax distance: {}", dist);
        println!("Bnf grammar: {}", trs_ltxnode.to_ebnf());
        if dist < distmin {
            distmin = dist;
            trs_chunk = trs_try;
        }
        // if distmin > 0 {
        //     // prepend a warning to the translation
        //     let warn = format!("%Warning chunk, distance: {}", distmin);
        //     trs_chunk = warn + trs_chunk.as_str();
        //     let endwarn = format!("%---------------------------------");
        //     trs_chunk = trs_chunk + endwarn.as_str();
        // }
        iter += 1;
    }

    Ok(trs_chunk)
}

// test the chat_with_ts function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_with_ts() {
        let question = "Q: Is Madrid the capital of Spain ?\nA:";
        let answer = chat_with_ts(question).unwrap();
        println!("{:?}", answer);
        assert!(answer.contains("Madrid"));
    }
    #[test]
    fn test_complete_grammar_ts() {
        let question = "Q: Is Tokyo the capital of Spain ?\nA:\n";
        let grammar = r#"root   ::= "yes" | "no""#;
        let grammar = grammar.to_string();
        println!("{:?}", grammar);
        let answer = complete_with_ts(question, &Some(grammar), "mistral47b".to_string()).unwrap();
        //let answer = complete_with_ts(question, None);
        println!("{:?}", answer);
        assert!(answer.contains("No") || answer.contains("no"));
    }
    #[test]
    fn test_2complete_grammar_ts() {
        let question = r#"
Question: 
What is the capital of France?
Give a false answer.

Answer:

"#;
        let grammar = r#"root   ::= [A-Z][a-z]*"#;
        let grammar = grammar.to_string();
        println!("{:?}", grammar);
        let answer = complete_with_ts(question, &Some(grammar),"mistral47b".to_string()).unwrap();
        // let answer = complete_with_ts(question, None);
        println!("{:?}", answer);
    }

    #[test]
    fn test_translate_with_grammar() {
        // prompt in the file "test/trs_sample_gram.txt"
        let prompt =
            std::fs::read_to_string("test/trs_sample_gram.txt").expect("cannot read prompt");
        // grammar in "src/sample.ebnf"
        let grammar = std::fs::read_to_string("src/sample.ebnf").expect("cannot read grammar");
        let str = complete_with_ts(&prompt, &None, "mistral47b".to_string()).unwrap();
        // print str in the terminal with true newlines
        println!("No grammar -------------------------------------------");
        let parts = str.split("\\n");
        for part in parts {
            println!("{}", part);
        }

        let str = complete_with_ts(&prompt, &Some(grammar), "mistral47b".to_string()).unwrap();
        // print str in the terminal with true newlines
        println!("With grammar -------------------------------------------");
        let parts = str.split("\\n");
        for part in parts {
            println!("{}", part);
        }
    }
}
