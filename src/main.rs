use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;
use askama::Template;
use logos::Logos;

use bitis_lib::*;

// use std::env;
use std::process::abort;

use clap::{Parser, Subcommand, ValueEnum};


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    input_file: PathBuf,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Language {
    /// use rust code
    Rust,
}
#[derive(Subcommand)]
enum Commands {
    /// Test bitis data objects file
    Test {},
    /// Compile bitis data objects file
    Compile {
        /// compile language
        #[arg(short, long)]
        lang: Language,
        /// output file
        #[arg(short, long)]
        output_file: PathBuf,
    },
    /// Compare bitis data objects file
    Compare {
        /// file to compare to input_file
        #[arg(short, long)]
        compare_file: PathBuf,
    },
}



fn main() {
    let cli = Cli::parse();

    // let content = fs::read_to_string("test.bitis").expect("File not found");
    let content = fs::read_to_string(cli.input_file).expect("Input File not found");

    let mut lexer = Token::lexer(content.as_str());

    let parsed_bitis = match parse_root(&mut lexer) {
        Ok(v) => v,
        Err(e) => {
            let (err_str, err_span) = e.clone();
            let content_err = &content[err_span];
            println!("Error: {}\n  -> Source: '{}'", err_str, content_err);
            abort()
        }
    };
    println!("{:?}", parsed_bitis);

    // ******
    match cli.command {
        Commands::Test {} => {
            if let Some(err_msg) = validate_bitis(&parsed_bitis) {
                println!("\nError: {}", err_msg);
            }
            else {
                println!("Ok!");
            }
        }
        Commands::Compile{lang, output_file} => {
            match lang {
                Language::Rust => {
                    let rdo = RustDataObjects {
                        enums: parsed_bitis.iter().filter_map(|x|
                            match x {
                                Value::Enum(ev) => Some((ev.name.clone(), ev.clone())),
                                _ => None
                            })
                            .collect::<HashMap<_, _>>(),
                        msgs: parsed_bitis.iter().filter_map(|x|
                            match x {
                                Value::Message(mv) => Some((mv.name.clone(), mv.clone())),
                                _ => None
                            })
                            .collect::<HashMap<_, _>>(),
                    };
                    let rendered = rdo.render().unwrap();
                    println!("{}", rendered);
                    fs::write(output_file, rendered).expect("Unable to write file");
                }
            }
        }
        Commands::Compare{ compare_file: _compare_file } => {
            println!("\n*** Compare not implemented yet\n");
        }
    }
}
