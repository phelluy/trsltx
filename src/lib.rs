//! # trsltx
//! Tools for automatic translation of texts written with LaTeX.
//!
//!  You need first to get a valid API key from [https://textsynth.com/](https://textsynth.com/)
//! and put it in a file named `api_key.txt` in the working directory or in an environment variable by
//!
//!  ```bash
//! export TEXTSYNTH_API_KEY=<the_api_key>
//! ```
//!
//!  Usage: go in the `trsltx` directory and run
//!
//! ```bash
//! cargo run
//! ```
//!
//! By default the French LaTeX file `test/simple_fr.tex` is translated into english in `test/simple_en.tex`.
//!
//! The languages are specified in the filename by the `_xy` mark, where `xy` is the abbreviated language name.
//! Currently, the available languages are: `en`, `fr`, `es`, `de`, `it`, `pt`, `ru`.
//!
//! For changing the default behavior do, for instance
//!
//! ```bash
//! cargo run -- -i test/simple_fr.tex -o test/simple_de.tex
//! ```
//!
//! Or
//!
//! ```bash
//! cargo build --release
//! cp ./target/release/trsltx .
//! ./trsltx -i test/simple_fr.tex -o test/simple_de.tex
//! ```
//!
//! The translation is done with a Large Language Model (LLM). It is possible that some LateX errors occur
//! in the translation by the LLM. You have to correct them by hand.use std::io::Write;

// For debug: useful for stopping the program with exit(0)
#[allow(unused_imports)]
use std::process::exit;

use std::io::Write;

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
    ) -> Trsltx {
        Trsltx {
            input_lang: input_lang.to_string(),
            output_lang: output_lang.to_string(),
            input_file_name: input_file_name.to_string(),
            output_file_name: output_file_name.to_string(),
            preamble: String::new(),
            body: String::new(),
            afterword: String::new(),
            body_translated: String::new(),
            chunks: Vec::new(),
        }
    }
    pub fn read_file(&mut self) {
        let input_file = std::fs::read_to_string(&self.input_file_name).expect("cannot read file");
        let mut input_file = input_file.split("\\begin{document}");
        self.preamble = input_file
            .next()
            .expect("Error in extracting body.")
            .to_string();
        let mut input_file = input_file
            .next()
            .expect("No \\begin{document} in the tex file.")
            .split("\\end{document}");
        self.body = input_file
            .next()
            .expect("Error in extracting body.")
            .to_string();
        self.afterword = input_file
            .next()
            .expect("No \\end{document} in the tex file.")
            .to_string();
    }

    pub fn translate(&mut self) {
        // self.body_translated = translate_chunk(
        //     self.body.as_str(),
        //     self.input_lang.as_str(),
        //     self.output_lang.as_str(),
        // );
        self.translate_chunks();
    }

    // extract the chunks to be translated from the body
    // the chunks are separated by the string "%trsltx-split\n"
    // or are enclosed between "%trsltx-begin-ignore\n" and "%trsltx-end-ignore\n"
    // by defaults, the chunks are marked as Translate
    // the chunks enclosed between "%trsltx-begin-ignore\n" and "%trsltx-end-ignore\n"
    // are marked as Unchanged
    pub fn extract_chunks(&mut self) {
        let toscan = self.body.clone();
        // add %trsltx-split before each %trsltx-begin-ignore
        let toscan = toscan.replace(
            "%trsltx-begin-ignore",
            "%trsltx-split\n%trsltx-begin-ignore",
        );
        // add %trsltx-split after each %trsltx-end-ignore
        let toscan = toscan.replace("%trsltx-end-ignore", "%trsltx-end-ignore\n%trsltx-split");
        // split the body into chunks
        let chunks = toscan.split("%trsltx-split\n");
        for chunk in chunks {
            if chunk.contains("%trsltx-begin-ignore") {
                assert!(
                    chunk.contains("%trsltx-end-ignore"),
                    "Ignored chunks cannot be split."
                );
                self.chunks.push((chunk.to_string(), ChunkType::Unchanged));
            } else if chunk.contains("%trsltx-end-ignore") {
                panic!("Unbalanced %trsltx-end-ignore");
            } else {
                self.chunks.push((chunk.to_string(), ChunkType::Translate));
            }
        }

        let numchunks = self.chunks.len();

        for i in 0..numchunks {
            let (s, t) = self.chunks[i].clone();
            let chunk_length = s.len();
            assert!(chunk_length < 1000, "Chunk too long");
            match t {
                ChunkType::Unchanged => {
                    // s = s.replace("%trsltx-begin-ignore\n", "");
                    // s = s.replace("%trsltx-end-ignore\n", "");
                    self.chunks[i] = (s, ChunkType::Unchanged);
                }
                _ => (),
            }
        }
        println!("{:?}", self.chunks);
    }

    pub fn translate_chunks(&mut self) {
        let mut body_translated = String::new();
        let numchunks = self.chunks.len();
        let mut count = 0;
        for (chunk, t) in self.chunks.iter() {
            match t {
                ChunkType::Translate => {
                    count += 1;
                    println!("Translating chunk {} of {}", count, numchunks);
                    let trs_chunk = translate_one_chunk(
                        chunk.as_str(),
                        self.input_lang.as_str(),
                        self.output_lang.as_str(),
                    );
                    // append the split message
                    if count > 1 {
                        body_translated.push_str("%trsltx-split\n");
                    }
                    body_translated.push_str(trs_chunk.as_str());
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

    pub fn write_file(&self) {
        let mut output_file =
            std::fs::File::create(&self.output_file_name).expect("cannot create file");
        output_file
            .write_all(self.preamble.as_bytes())
            .expect("cannot write to file");

        // write the translated body
        // create the latex env trsltx because the prompt
        // requires that the translatex chunk is enclosed between
        // \begin{trsltx} and \end{trsltx}
        output_file
            .write_all("\\newenvironment{trsltx}{}{}\n\\begin{document}".as_bytes())
            .expect("cannot write to file");

        output_file
            .write_all(self.body_translated.as_bytes())
            .expect("cannot write to file");
        output_file
            .write_all("\\end{document}".as_bytes())
            .expect("cannot write to file");
        output_file
            .write_all(self.afterword.as_bytes())
            .expect("cannot write to file");
    }
}

/// Get the long language name from the short two-letter one
pub fn get_lang_name(lang: &str) -> String {
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

    let lang = lang_dict.get(lang).expect("Unknown language");
    lang.to_string()
}

/// one chat operation with the textsynth LLM
/// send the question
/// and returns an answer
fn chat_with_ts(question: &str) -> String {
    // get the api key from the file "api_key.txt" or if the file does not exist, from the environment variable "TEXTSYNTH_API_KEY"
    let api_key = match std::fs::read_to_string("api_key.txt") {
        // if the file exists, get the api key from the file
        // removing the spaces and newlines with trim()
        Ok(api_key) => api_key.trim().to_string(),
        Err(_) => std::env::var("TEXTSYNTH_API_KEY").expect("You have to provide an api key in the file api_key.txt or by export TEXTSYNTH_API_KEY=api_key"),
    };

    // call the textsynth REST API
    let url = "https://api.textsynth.com/v1/engines/mixtral_47B_instruct/chat";
    let max_tokens = 1000;

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
            let text = resp["text"].as_str().expect("Failed to get text");
            //println!("{:?}", text);
            text.to_string()
        }
        Err(e) => {
            println!("Request error: {:?}", e);
            "".to_string()
        }
    };

    answer
}

/// one completion operation with the textsynth LLM
/// send the question and a formal grammar (as Some(String) or None)
/// and returns an answer
fn complete_with_ts(prompt: &str, grammar: Option<String>) -> String {
    // get the api key from the file "api_key.txt" 
    //or if the file does not exist, from the environment variable "TEXTSYNTH_API_KEY"
    let api_key = match std::fs::read_to_string("api_key.txt") {
        // if the file exists, get the api key from the file
        // removing the spaces and newlines with trim()
        Ok(api_key) => api_key.trim().to_string(),
        Err(_) => std::env::var("TEXTSYNTH_API_KEY").expect("You have to provide an api key in the file api_key.txt or by export TEXTSYNTH_API_KEY=api_key"),
    };

    // call the textsynth REST API
    let url = "https://api.textsynth.com/v1/engines/mixtral_47B_instruct/completions";
    let max_tokens = 1000;

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
    // println!("Req= {:?}", req);

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
            println!("{:?}", resp);
            let text = resp["text"].as_str().expect("Failed to get text");
            //println!("{:?}", text);
            text.to_string()
        }
        Err(e) => {
            println!("Request error: {:?}", e);
            "".to_string()
        }
    };

    answer
}

/// translate a latex chunk using the textsynth LLM api
/// the preprompt is in the file "prompt.txt"
/// the api key is in the file "api_key.txt" or
/// in the environment variable "TEXTSYNTH_API_KEY"
fn translate_one_chunk(chunk: &str, input_lang: &str, output_lang: &str) -> String {
    // get the preprompt
    let mut prompt = std::fs::read_to_string("src/prompt.txt").expect("cannot read preprompt");

    let input_lang = get_lang_name(input_lang).to_string();
    let output_lang = get_lang_name(output_lang).to_string();

    // in the prompt, replace <lang_in> by the input language and <lang_out> by the output language
    prompt = prompt.replace("<lang_in>", input_lang.as_str());
    prompt = prompt.replace("<lang_out>", output_lang.as_str());

    let question = format!("{}{}", prompt, chunk);
    let trs_chunk = chat_with_ts(question.as_str());

    // remove the text before \begin{trsltx} and after \end{trsltx}
    let trs_chunk = trs_chunk.split("\\begin{trsltx}").collect::<Vec<&str>>()[1];
    let trs_chunk = trs_chunk.split("\\end{trsltx}").collect::<Vec<&str>>()[0];
    trs_chunk.to_string()
}

// test the chat_with_ts function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_with_ts() {
        let question = "What is the capital of France?";
        let answer = chat_with_ts(question);
        println!("{:?}", answer);
        assert!(answer.contains("Paris"));
    }
    #[test]
    fn test_complete_grammar_ts() {
        let question = "Q: la capitale de la France est-elle Paris? Répondre uniquement par oui ou non avec des caractères en minuscule.\nA:";
        let grammar = 
r#"root   ::= "oui" | "non""#;
        let grammar = grammar.to_string();
        println!("{:?}", grammar);
        let answer = complete_with_ts(question, Some(grammar));
        println!("{:?}", answer);
    }
}
