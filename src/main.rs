//# trsltx
//Tools for automatic translation of texts written with LaTeX.use clap::Parser;
//The Rust doc is in the `src/lib.rs` file.

//use std::path;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, default_value = "test/simple.tex")]
    file_init: String,
    #[clap(short, long, default_value = "fr")]
    input_lang: String,
    #[clap(short, long, default_value = "en")]
    output_lang: String,
    #[clap(short, long, default_value = "2000")]
    length_split: usize,
}

use trsltx::Trsltx;

// init_file: the tex file to be translated
// input_lang: the language of the input file
// output_lang: the language of the output file
// input_file: init_file with an addition _xy.tex suffix where xy is the input language
// output_file: init_file with an addition _zt.tex suffix where zt is the output language
// if the input file does not exist, it is created with the content of the init file
// and additional markers for splitting the file into chunks
// if the input file exists, it is not modified by the command
fn main() -> Result<(), String> {
    let args = Cli::parse();
    let init_file = args.file_init.as_str();
    let in_lang = args.input_lang.as_str();
    let in_lang = format!("_{}.tex", in_lang);
    let input_file = init_file.replace(".tex", &in_lang).clone();
    let out_lang = args.output_lang.as_str();
    let out_lang = format!("_{}.tex", out_lang);
    let output_file = init_file.replace(".tex", &out_lang).clone();

    let init_file_name = init_file;
    let input_file_name = input_file.clone();
    let output_file_name = output_file.clone();

    // asserts that the filenames end with .tex
    assert!(input_file.ends_with(".tex"));
    assert!(output_file.ends_with(".tex"));

    // removes the .tex suffix
    // before check that length is > 4
    assert!(input_file.len() > 4);
    assert!(output_file.len() > 4);
    let input_file = &input_file[..input_file.len() - 4];
    let output_file = &output_file[..output_file.len() - 4];

    // split the string at the _
    let input_file: Vec<&str> = input_file.split('_').collect();
    let output_file: Vec<&str> = output_file.split('_').collect();

    assert!(input_file.len() == 2);
    assert!(output_file.len() == 2);

    let input_lang = input_file[1];
    let output_lang = output_file[1];

    if output_lang == input_lang {
        return Err("The source and target languages are the same".to_string());
    }

    // if the input file does not exist read the init file, split it and write it to the input file
    let path_to_file = std::path::Path::new(&input_file_name);
    println!("{},path_to_file={:?}", input_file_name, path_to_file);
    //assert!(1==2);
    if !path_to_file.exists() {
        // read init_file
        println!("Reading input file {}", input_file_name);
        //let s = std::fs::read_to_string(init_file_name).map_err(|e| e.to_string())?;

        let mut trsltx = Trsltx::new(input_lang, output_lang, init_file_name, "");
        trsltx.read_file()?;
        println!("{:?}", trsltx);
        let s = trsltx.generate_split_latex(args.length_split);

        // save to input_file
        println!("Writing input file {}", input_file_name);
        std::fs::write(&input_file_name, s).map_err(|e| e.to_string())?;
    }
    let mut trsltx = Trsltx::new(
        input_lang,
        output_lang,
        input_file_name.as_str(),
        output_file_name.as_str(),
    );

    trsltx.read_file()?;
    trsltx.extract_chunks()?;
    trsltx.translate();
    trsltx.write_file()?;

    Ok(())
}
