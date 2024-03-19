//# trsltx
//Tools for automatic translation of texts written with LaTeX.use clap::Parser;
//The Rust doc is in the `src/lib.rs` file.


use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, default_value = "test/simple_fr.tex")]
    input_file: String,
    #[clap(short, long, default_value = "test/simple_en.tex")]
    output_file: String,
}

use trsltx::Trsltx;

fn main() {
    let args = Cli::parse();
    let input_file = args.input_file.as_str();
    let output_file = args.output_file.as_str();

    let input_file_name = input_file;    
    let output_file_name = output_file;


    // asserts that the filenames end with .tex
    assert!(input_file.ends_with(".tex"));
    assert!(output_file.ends_with(".tex"));

    // removes the .tex suffix
    // before check that length is > 4
    assert!(input_file.len() > 4);
    assert!(output_file.len() > 4);
    let input_file = &input_file[..input_file.len()-4];
    let output_file = &output_file[..output_file.len()-4];

    // split the string at the _
    let input_file : Vec<&str> = input_file.split('_').collect();
    let output_file : Vec<&str> = output_file.split('_').collect();

    assert!(input_file.len() == 2);
    assert!(output_file.len() == 2);

    let input_lang = input_file[1];
    let output_lang = output_file[1];

    let mut trsltx = Trsltx::new(input_lang, output_lang, input_file_name, output_file_name);

    trsltx.read_file();
    trsltx.translate();
    trsltx.write_file();

}
