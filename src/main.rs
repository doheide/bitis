use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use askama::Template;
use logos::Logos;

use bitis_lib::*;

// use std::env;
use std::process::abort;

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;

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
    let input_file_path = Path::new(&cli.input_file);
    if !input_file_path.exists() {
        panic!("Input file {:?} does not exist.", input_file_path);
    }
    if input_file_path.extension().unwrap() != "bitis" {
        panic!("Input file extension needs to be 'bitis'.");
    }
    let input_dir = input_file_path.parent().unwrap();
    let input_dir = if let None = input_dir.parent() { PathBuf::from("./") } else { input_dir.to_owned() };
    let inout_file_wo_ext = input_file_path.file_stem().unwrap();

    println!("Input file: {:?} (dir: {})", input_file_path, input_dir.to_str().unwrap());

    // println!("input_dir: {input_dir:?}");
    // println!("inout_file_wo_ext: {inout_file_wo_ext:?}");

    let re = Regex::new(r".+\.v([0-9]+)").unwrap();
    let ver_files: Vec<_> = fs::read_dir(input_dir).unwrap().into_iter().filter_map(|x| {
        let cf = x.unwrap().path();
        let cf_stem = cf.file_stem().unwrap();
        let v = match re.captures(&cf_stem.to_str().unwrap()) {
            Some(v) => match v.get(1) {
                Some(vv) => { vv.as_str().parse::<u16>().unwrap() }, None => 0 }, None => 0 };
        if cf_stem != inout_file_wo_ext &&
            v == 0 { None }
        else if cf.extension().unwrap() != "bitis" { None }
        else { Some((cf, v)) }
    }).collect();
    println!("Inputs version files: {:?}", ver_files.iter().map(|x| x).collect::<Vec<_>>());

    let parsed_bitis: Vec<_> = ver_files.iter().map(|f|
        {
            let content = fs::read_to_string(&(f.0)).expect("Input File not found");
            let mut lexer = Token::lexer(content.as_str());
            lexer.extras = f.1;
            println!("file: {} ver: {}", f.0.to_str().unwrap(), f.1);

            match parse_root(&mut lexer) {
                Ok(v) => v,
                Err(e) => {
                    let (err_str, err_span) = e.clone();
                    let content_err = &content[err_span];
                    println!("Error: {}\n  -> Source: '{}'", err_str, content_err);
                    abort()
                }
            }
        }).flatten().collect();
    println!("{:?}", parsed_bitis);

    // ******
    match cli.command {
        Commands::Test {} => {
            if let Some(err_msg) = validate_bitis(&parsed_bitis) {
                println!("\nError: {}", err_msg);
            } else {
                println!("Ok!");
            }
        }
        Commands::Compile { lang, output_file } => {
            match lang {
                Language::Rust => {
                    let rdo = RustDataObjects {
                        enums: parsed_bitis.iter().filter_map(|x|
                            match x {
                                Value::Enum(ev) => {
                                    let n = match &ev.version {
                                        MsgVersion::FixedMsg => ev.name.clone(),
                                        MsgVersion::VersionedMsg(v) => {
                                            format!("{}_v{}", ev.name.clone(),v.to_string())
                                        }
                                    };
                                    Some((n, ev.clone()))
                                },
                                _ => None
                            })
                            .collect::<HashMap<_, _>>(),
                        msgs: parsed_bitis.iter().filter_map(|x|
                            match x {
                                Value::Message(mv) => {
                                    let n = match &mv.version {
                                        MsgVersion::FixedMsg => mv.name.clone(),
                                        MsgVersion::VersionedMsg(v) => {
                                            format!("{}_v{}", mv.name.clone(),v.to_string())
                                        }
                                    };
                                    Some((n, mv.clone())) },
                                _ => None
                            })
                            .collect::<HashMap<_, _>>(),
                    };
                    println!("rdo: {:?}", rdo.clone());

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
