// cli tool for using the trsltx library
// the utility takes two optionnal arguments:
// - the input filename. By default it is article_fr.tex
// the _fr suffix is used to indicate the language of the file (here french)
// - the output filename. By default it is article_en.tex
// the _en suffix is used to indicate the language of the file (here english) 

// needed includes for handling arguments of the command line
use std::env;

use trsltx::Trsltx;




fn main() {
    let args : Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 { &args[1] } else { "article_fr.tex" };
    let output_file = if args.len() > 2 { &args[2] } else { "article_en.tex" };

    let input_file_name = input_file.clone();    
    let output_file_name = output_file.clone();


    // asserts that the filenames end with .tex
    assert!(input_file.ends_with(".tex"));
    assert!(output_file.ends_with(".tex"));

    // removes the .tex suffix
    let input_file = &input_file[..input_file.len()-4];
    let output_file = &output_file[..output_file.len()-4];

    // split the string at the _
    let input_file : Vec<&str> = input_file.split('_').collect();
    let output_file : Vec<&str> = output_file.split('_').collect();

    assert!(input_file.len() == 2);
    assert!(output_file.len() == 2);

    let input_lang = input_file[1];
    let output_lang = output_file[1];

    // println!("input language: {:?}", input_lang);
    // println!("input file name: {:?}", input_file_name);
    // println!("output language: {:?}", output_lang);
    // println!("output file name: {:?}", output_file_name);

    let mut trsltx = Trsltx::new(input_lang, output_lang, input_file_name, output_file_name);

    trsltx.read_file();
    trsltx.write_file();

    //println!("{:?}", trsltx);

}
