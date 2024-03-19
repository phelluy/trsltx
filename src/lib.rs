// simple library for managing translations of latex files
use std::io::Write;


//use std::collections::HashMap;
// unused warning
//#[allow(unused_imports)]
//use std::process::exit;

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
        }
    }
    pub fn read_file(&mut self) {
        let input_file = std::fs::read_to_string(&self.input_file_name).expect("cannot read file");
        let mut input_file = input_file.split("\\begin{document}");
        self.preamble = input_file.next().unwrap().to_string();
        let mut input_file = input_file.next().unwrap().split("\\end{document}");
        self.body = input_file.next().unwrap().to_string();
        self.afterword = input_file.next().unwrap().to_string();
        //println!("{:?}", self.afterword);
    }

    pub fn write_file(&self) {
        let mut output_file =
            std::fs::File::create(&self.output_file_name).expect("cannot create file");
        output_file
            .write_all(self.preamble.as_bytes())
            .expect("cannot write to file");
        output_file
            .write_all("\\newenvironment{trsltx}{}{}\n\\begin{document}".as_bytes())
            .expect("cannot write to file");
        let body_out = translate_chunk(
            self.body.as_str(),
            self.input_lang.as_str(),
            self.output_lang.as_str(),
        );
        output_file
            .write_all(body_out.as_bytes())
            .expect("cannot write to file");
        output_file
            .write_all("\\end{document}".as_bytes())
            .expect("cannot write to file");
        output_file
            .write_all(self.afterword.as_bytes())
            .expect("cannot write to file");
    }
}

// translate a latex chunk using the textsynth LLM api
// the prepromt is in the fille "prompt.txt"
fn translate_chunk(chunk: &str, input_lang: &str, output_lang: &str) -> String {
    // get the preprompt
    let mut prompt = std::fs::read_to_string("src/prompt.txt").expect("cannot read file");

    // list of known languages: en, fr, es, de, it, pt, ru
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

    let input_lang = lang_dict.get(input_lang).unwrap();
    let output_lang = lang_dict.get(output_lang).unwrap();

    // in this prompt, replace <lang_in> by the input language and <lang_out> by the output language
    prompt = prompt.replace("<lang_in>", input_lang);
    prompt = prompt.replace("<lang_out>", output_lang);

    //println!("{:?}", prompt);

    // get the api key from the file "api_key.txt"
    let api_key = std::fs::read_to_string("api_key.txt").expect("cannot read file");
    //println!("{:?}", api_key);

    let url = "https://api.textsynth.com/v1/engines/falcon_40B-chat/chat";
    let max_tokens = 1000;

    use serde_json::json;
    use serde_json::Value;
    use std::process;
    let question = format!("{}{}", prompt, chunk);
    println!("{:?}", question);
    let req = json!({
        "messages": [question],
        "temperature": 0.5,
        "max_tokens": max_tokens
    });
    // println!("{:?}", serde_json::to_string(&req).unwrap());
    // exit(0);
    // let client = reqwest::blocking::Client::new();
    // let res = client.post(&format!("{}/v1/engines/{}/chat", url, model))
    //     .header(AUTHORIZATION, format!("Bearer {}", api_key))
    //     .json(&req);

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .expect("Failed to send request")
        .json::<Value>();


    //let res = res.send();

    let mut trs_chunk = String::new();

    match res {
        Ok(resp) => {
            //println!("Réponse: {:?}", resp);
            let text = resp["text"].as_str().expect("Failed to get text");
            // let text: String = resp.json().unwrap();
            println!("{:?}", text);
            trs_chunk = text.to_string();
        }
        Err(e) => {
            println!("Request error: {:?}", e);
            process::exit(1);
        }
    }

    trs_chunk
}
