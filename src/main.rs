mod utils;

use std::fs;
use std::fs::write;
use std::path::{Path, PathBuf};
use askama::Template;
use logos::Logos;

use bitis_lib::*;

// use std::env;
use std::process::{abort, exit, Command, Stdio};

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use toml_edit::{value, DocumentMut};
use utils::*;


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
    Python,
    Cpp
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum System {
    /// maturin framework to build python packages from rust
    Maturin,
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
        output_file_or_path: Option<PathBuf>,
    },
    /// Compare bitis data objects file
    Compare {
        /// file to compare to input_file
        #[arg(short, long)]
        compare_file: PathBuf,
    },
    /// Setup directory and file structures
    Setup {
        /// system to set up
        #[arg(short, long)]
        system: System,
        output_path: PathBuf,
    }
}



fn main() {
    let cli = Cli::parse();

    // let content = fs::read_to_string("test.bitis").expect("File not found");
    let input_file_path = Path::new(&cli.input_file);
    if !input_file_path.exists() {
        println!("Input file {:?} does not exist.", input_file_path); abort();
    }
    if input_file_path.extension().unwrap() != "bitis" {
        println!("Input file extension needs to be 'bitis'."); abort()
    }
    let input_dir = input_file_path.parent().unwrap();
    let input_dir = if let None = input_dir.parent() { PathBuf::from("./") } else { input_dir.to_owned() };
    let input_file_wo_ext = input_file_path.file_stem().unwrap();

    if cli.debug > 0 { println!("Input file: {:?} (dir: {})", input_file_path, input_dir.to_str().unwrap()); }

    // println!("input_dir: {input_dir:?}");
    // println!("inout_file_wo_ext: {inout_file_wo_ext:?}");

    let re = Regex::new(r".+\.v([0-9]+)").unwrap();
    let ver_files: Vec<_> = fs::read_dir(input_dir.clone()).unwrap().into_iter().filter_map(|x| {
        let cf = x.unwrap().path();
        let cf_stem = cf.file_stem().unwrap();
        let v = match re.captures(&cf_stem.to_str().unwrap()) {
            Some(v) => match v.get(1) {
                Some(vv) => { vv.as_str().parse::<u16>().unwrap() }, None => 0 }, None => 0 };
        if cf_stem != input_file_wo_ext &&
            v == 0 { None }
        else if cf.extension().unwrap() != "bitis" { None }
        else { Some((cf, v)) }
    }).collect();
    if cli.debug > 0 { println!("Inputs version files: {:?}", ver_files.iter().map(|x| x).collect::<Vec<_>>()); }

    let parsed_bitis: Vec<_> = ver_files.iter().map(|f| {
        let content = fs::read_to_string(&(f.0)).expect("Input File not found");
        let mut lexer = Token::lexer(content.as_str());
        lexer.extras = f.1;
        if cli.debug > 3 { println!("file: {} ver: {}", f.0.to_str().unwrap(), f.1); }
        if cli.debug > 3 { println!("bitis-file content:\n{}", content); }
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
    if cli.debug > 1 { println!("parsed_bitis: {:?}", parsed_bitis); }

    let processed_bitis = process_and_validate_bitis(&parsed_bitis);
    if cli.debug > 2 { println!("processed_bitis: {:?}", processed_bitis); }

    // ******
    match cli.command {
        Commands::Test {} => {
        }
        Commands::Compile { lang, output_file_or_path: output_file_opt } => {
            match lang {
                Language::Rust => {
                    let output_file = if let Some(output_file_opt_set) = output_file_opt {
                        if output_file_opt_set.is_dir() {
                            let mut of = output_file_opt_set.clone();
                            of.push(format!("{}.rs", input_file_wo_ext.to_str().unwrap()).as_str());
                            of
                        }
                        else { output_file_opt_set }
                    }
                    else{
                        let mut pb = PathBuf::new();
                        pb.push(input_dir.clone().to_str().unwrap());
                        pb.push(format!("{}.rs", input_file_wo_ext.to_str().unwrap()).as_str());
                        pb
                    };
                    let rdo = RustDataObjects{ d: JinjaData{enums: processed_bitis.enums,
                        msgs: to_rust_messages(&processed_bitis.msgs),
                        oos: to_rust_oneofs(&processed_bitis.oo_enums,&processed_bitis.msgs) } };

                    let rendered = rdo.render().unwrap();
                    println!("{}", rendered);
                    fs::write(output_file, rendered).expect("Unable to write file");
                },
                Language::Python => {
                    if output_file_opt.is_none() {
                        println!("Error: Output path has to be set for python language compiler.");
                        exit(-1);
                    }
                    let output_path = output_file_opt.unwrap();
                    if !output_path.is_dir() {
                        println!("Error: Output path has to be a directory for python language compiler.")
                    }
                    let lib_name = match output_path.file_name() {
                        Some(v) => v.to_str().unwrap(), None => {
                            println!("Error: Output path has to consist of the lib-name."); exit(-1); }
                    };

                    if cli.debug > 0 { println!("* Lib-name: {}\n", lib_name); }

                    if !{ let mut t = output_path.clone(); t.push(lib_name); t }.is_dir() {
                        println!("The python lib seems not to be setup correctly: Expected subdir '{}' in output path ({})",
                                 lib_name, output_path.to_str().unwrap());
                        exit(-1);
                    }
                    if !{ let mut t = output_path.clone(); t.push("src"); t }.is_dir() {
                        println!("The python lib seems not to be setup correctly: Expected subdir 'src' in output path ({})", output_path.to_str().unwrap());
                        exit(-1);
                    }

                    let d = JinjaData{
                        enums: processed_bitis.enums,
                        msgs: to_rust_messages(&processed_bitis.msgs),
                        oos: to_rust_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs)
                    };

                    fn write_file(base_path: &PathBuf, file: &str, content: &str) {
                        let mut cp = base_path.clone();
                        cp.push(file);
                        if fs::write(cp.clone(), content).is_err(){
                            println!("Could not write file '{}'", cp.to_str().unwrap());
                        }
                    }
                    let rdo = RustDataObjects{ d: d.clone() };
                    let rendered_rust = rdo.render().unwrap();
                    write_file(&output_path, "src/messages.rs", rendered_rust.as_str());

                    let rdo = RustPyDataObjects{ d: d.clone() };
                    let rendered_rust = rdo.render().unwrap();
                    write_file(&output_path, "src/pyrust.rs", rendered_rust.as_str());

                    let rdo = RustPyLib{ d: d.clone(), lib_name: String::from(lib_name) };
                    let rendered_rust = rdo.render().unwrap();
                    write_file(&output_path, "src/lib.rs", rendered_rust.as_str());

                    let rdo = PyTypeHints{ d };
                    let rendered_rust = rdo.render().unwrap();
                    write_file(&output_path, format!("{}/bitis_msgs.pyi", lib_name).as_str(), rendered_rust.as_str());

                    let r = match Command::new("maturin").args(["develop"]).current_dir(output_path.clone())
                        .stdout(Stdio::piped()).spawn() {
                        Ok(v) => v, Err(_) => { println!("Could not execute 'maturin develop'"); exit(-1) }
                    };
                    let out = match r.wait_with_output() {
                        Ok(v) => v, Err(e) => { println!("Error waiting for 'maturin develop': {}", e); exit(-1) }
                    };
                    if !out.status.success() {
                        println!("Error: 'maturin develop' returned error {}'", String::from_utf8(out.stderr).unwrap());
                        exit(-1);
                    }
                    else { println!("\n🎉 * Bitis compile and python lib build was successfully executed!\n"); }

                    {
                        let toml_file = output_path.join("Cargo.toml");
                        if toml_file.exists() {
                            let cct_content = match fs::read(toml_file.clone()) {
                                Ok(content) => String::from_utf8(content).unwrap(),
                                Err(e) => { print_error(format!("Could not read lock file '{}': {:?}", toml_file.display(), e)); }
                            };
                            let mut toml_doc = match cct_content.parse::<DocumentMut>() {
                                Ok(v) => v, Err(e) => {
                                    print_error(format!("Could not parse toml from lock file '{}': {:?}", toml_file.display(), e)); }
                            };

                            toml_doc["dependencies"]["bitis"] = value(env!("CARGO_PKG_VERSION"));

                            if let Err(e) = write(toml_file.clone(), toml_doc.to_string()) {
                                print_error(format!("Failed to write to lock '{}': {}", toml_file.display(), e));
                            }
                        }
                        else { print_warn("There was no toml file found in the base directory.".into()) }
                    }
                }
                Language::Cpp => {
                    let output_file = if let Some(output_file_opt_set) = output_file_opt {
                        if output_file_opt_set.is_dir() {
                            let mut of = output_file_opt_set.clone();
                            of.push(format!("{}.h", input_file_wo_ext.to_str().unwrap()).as_str());
                            of
                        }
                        else { output_file_opt_set }
                    }
                    else{
                        let mut pb = PathBuf::new();
                        pb.push(input_dir.clone().to_str().unwrap());
                        pb.push(format!("{}.h", input_file_wo_ext.to_str().unwrap()).as_str());
                        pb
                    };
                    
                    let jd = JinjaData{enums: processed_bitis.enums,
                        msgs: to_cpp_messages(&processed_bitis.msgs),
                        oos: to_cpp_oneofs(&processed_bitis.oo_enums, &processed_bitis.msgs) };
                    let object_order = dependencies_process(jd.clone());
                    let rdo = CppDataObjects{ d: jd, object_order };
                    
                    let rendered = rdo.render().unwrap();
                    // println!("{}", rendered);
                    fs::write(output_file.clone(), rendered).expect("Unable to write file");
                    println!("Written to {}", output_file.to_str().unwrap());
                }
        } },
        Commands::Compare{ compare_file: _compare_file } => {
            println!("\n*** Compare not implemented yet\n");
        },
        Commands::Setup{system, output_path} => {
            match system {
                System::Maturin => {
                    let lib_name = match output_path.file_name() {
                        Some(v) => v.to_str().unwrap(), None => {
                            println!("Error: Output path has to be the future lib name."); exit(-1); }
                    };
                    if cli.debug > 1 { println!("* Lib-name: {}\n", lib_name); }

                    // check if venv is enabled
                    let r = match Command::new("pip").args(["-V"]).output() {
                        Ok(v) => v, Err(_) => { println!("Could not find pip executable"); exit(-1) }
                    };
                    if !String::from_utf8(r.stdout).unwrap().contains(".venv") {
                        println!("Venv needs to be activated for setting up pylib"); exit(-1)
                    }
                    // check for maturin
                    match Command::new("maturin").args(["--version"]).output() {
                        Ok(v) => v, Err(_) => { println!("Maturin python package not installed. Please install it with\n\n  pip install maturin\n"); exit(-1) }
                    };
                    // make dir
                    if !output_path.exists() {
                        if fs::create_dir_all(output_path.clone()).is_err() {
                            println!("Unable to create output directory {}", output_path.clone().to_str().unwrap());
                            exit(-1);
                        }
                    }
                    // if there is no src directory, call maturin init
                    if !{ let mut t = output_path.clone(); t.push("src"); t}.exists() {
                        if cli.debug > 1 { println!("Initializing maturin project ..."); }
                        let r = match Command::new("maturin").args(["init", "-b", "pyo3"])
                            .current_dir(output_path.clone().to_str().unwrap()).output() {
                            Ok(v) => v, Err(e) => { println!("'maturin init' failed: {}", e); exit(-1) }
                        };
                        if !r.status.success() {
                            println!("'maturin init' failed: {}", String::from_utf8(r.stderr).unwrap()); exit(-1)
                        }
                        println!("maturin response: {:?}", r);
                        if cli.debug > 1 { println!("  done!"); }
                    }
                    let mut py_code_dir = output_path.clone();
                    py_code_dir.push(lib_name);
                    if !py_code_dir.exists() {
                        if fs::create_dir_all(&py_code_dir).is_err() {
                            println!("Could not create directory '{}'", py_code_dir.to_str().unwrap()); exit(-1);
                        }
                        if fs::write({let mut t = py_code_dir.clone(); t.push("__init__.py"); t},
                                     format!("from .{} import *", lib_name).as_str()).is_err() {
                            println!("Could not create file '__init__.py' in dir '{}'", py_code_dir.to_str().unwrap()); exit(-1);
                        }
                        if fs::write({let mut t = py_code_dir.clone(); t.push("py.typed"); t}, "").is_err() {
                            println!("Could not create file 'py.typed' in dir '{}'", py_code_dir.to_str().unwrap()); exit(-1);
                        }
                    }
                    println!("*** Project successfully setup.");
                }
            }

        }
    }
}
