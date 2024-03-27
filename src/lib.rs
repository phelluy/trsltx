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
    pub fn read_file(&mut self) -> Result<(), String> {
        let input_file = std::fs::read_to_string(&self.input_file_name)
            .map_err(|e| format!("Cannot read file: {:?}", e))?;
        // replace \r characters by nothing (appear in Windows files...)
        let input_file = input_file.replace('\r', "");
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
        self.translate_chunks();
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
        let toscan = toscan.replace("%trsltx-end-ignore", "%trsltx-end-ignore\n%trsltx-split");
        // split the body into chunks
        let chunks = toscan.split("%trsltx-split\n");
        for chunk in chunks {
            if chunk.contains("%trsltx-begin-ignore") {
                if !chunk.contains("%trsltx-end-ignore") {
                    return Err("Unbalanced %trsltx-begin-ignore".to_string());
                }
                self.chunks.push((chunk.to_string(), ChunkType::Unchanged));
            } else if chunk.contains("%trsltx-end-ignore") {
                return Err("Unbalanced %trsltx-end-ignore".to_string());
            } else {
                self.chunks.push((chunk.to_string(), ChunkType::Translate));
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
                        )
                    };
                    match trs_try {
                        Ok(trs_chunk) => {
                            // append the split message
                            // so that the translated file
                            // can be reused by trsltx
                            if count > 1 {
                                body_translated.push_str("%trsltx-split\n");
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
            .write_all("\\newenvironment{trsltx}{}{}\n\\begin{document}".as_bytes())
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

/// Get the long language name from the short two-letter one
pub fn get_lang_name(lang: &str) -> Result<String, String> {
    // list of known languages xxx
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

    let lang = lang_dict.get(lang).ok_or("The supported languages are: en,fr,es,de,it,pt,ru. Unsupported language: ".to_owned()+&lang)?;
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
fn complete_with_ts(prompt: &str, grammar: Option<String>) -> Result<String, String> {
    // get the api key from the file "api_key.txt"
    //or if the file does not exist, from the environment variable "TEXTSYNTH_API_KEY"
    let api_key = match std::fs::read_to_string("api_key.txt") {
        // if the file exists, get the api key from the file
        // removing the spaces and newlines with trim()
        Ok(api_key) => api_key.trim().to_string(),
        Err(_) => std::env::var("TEXTSYNTH_API_KEY").map_err(|e| format!("You have to provide an api key in the file api_key.txt or by export TEXTSYNTH_API_KEY=api_key. Error: {:?}", e))?,
    };

    // call the textsynth REST API
    let url = "https://api.textsynth.com/v1/engines/mixtral_47B_instruct/completions";
    //let url = "https://api.textsynth.com/v1/engines/llama2_70B/completions";

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
    //println!("Req= {:?}", req);

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

mod ltxprs;

use ltxprs::LtxNode;

/// translate a latex chunk using the textsynth LLM api
/// the preprompt is in the file "prompt.txt"
/// the api key is in the file "api_key.txt" or
/// in the environment variable "TEXTSYNTH_API_KEY"
fn translate_one_chunk(chunk: &str, input_lang: &str, output_lang: &str) -> Result<String, String> {
    // get the preprompt
    let mut prompt = std::fs::read_to_string("src/prompt.txt")
        .map_err(|_| "cannot read preprompt".to_string())?;

    let input_lang = get_lang_name(input_lang)?.to_string();
    let output_lang = get_lang_name(output_lang)?.to_string();

    // in the prompt, replace <lang_in> by the input language and <lang_out> by the output language
    prompt = prompt.replace("<lang_in>", input_lang.as_str());
    prompt = prompt.replace("<lang_out>", output_lang.as_str());

    let question = format!("{}\n{}\nA:\n", prompt, chunk);
    // println!("{:?}", question);
    // exit(0);
    //let trs_chunk = chat_with_ts(question.as_str());
    let ast_chunk = LtxNode::new(chunk);
    //let cmds = ast_chunk.extracts_commands();
    //println!("{:?}", ast_chunk);
    let grammar = match ast_chunk {
        LtxNode::None => None,
        _ => Some(ast_chunk.to_ebnf().trim().to_string()),
    };
    println!("Grammar: {}", ast_chunk.to_ebnf());
    let trs_try = complete_with_ts(question.as_str(), grammar);
    //let trs_chunk = complete_with_ts(&question.as_str(), None);

    // remove the text before \begin{trsltx} and after \end{trsltx}
    // if they exist, do nothing if they do not exist
    let trs_chunk = match trs_try {
        Ok(trs_chunk) => trs_chunk,
        Err(e) => return Err(e),
    };
    let trs_chunk = trs_chunk.split("\\begin{trsltx}").collect::<Vec<&str>>();
    Ok(if trs_chunk.len() >= 2 {
        let trs_chunk = trs_chunk[1].split("\\end{trsltx}").collect::<Vec<&str>>()[0];
        trs_chunk.to_string()
    } else {
        "".to_string()
    })
    // let trs_chunk = trs_chunk.split("\\begin{trsltx}").collect::<Vec<&str>>()[1];
    // let trs_chunk = trs_chunk.split("\\end{trsltx}").collect::<Vec<&str>>()[0];
    // trs_chunk.to_string()
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
        let answer = complete_with_ts(question, Some(grammar)).unwrap();
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
        let answer = complete_with_ts(question, Some(grammar));
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
        let str = complete_with_ts(&prompt, None).unwrap();
        // print str in the terminal with true newlines
        println!("No grammar -------------------------------------------");
        let parts = str.split("\\n");
        for part in parts {
            println!("{}", part);
        }

        let str = complete_with_ts(&prompt, Some(grammar)).unwrap();
        // print str in the terminal with true newlines
        println!("With grammar -------------------------------------------");
        let parts = str.split("\\n");
        for part in parts {
            println!("{}", part);
        }
    }
}
