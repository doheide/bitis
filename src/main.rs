mod utils;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bitis_lib::*;

use clap::{crate_version, Parser, Subcommand, ValueEnum};
use toml_edit::{value, DocumentMut};
use utils::*;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on (can be given multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Language {
    /// generate rust code
    Rust,
    /// generate python library (based on generated rust code and maturin)
    Python,
    /// generate c++ code
    Cpp
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum System {
    /// maturin framework to build python packages from rust
    Maturin,
}
#[derive(Subcommand)]
enum Commands {
    /// Compile bitis definition files to code
    Compile {
        /// Input file(s) with extension .bitis; give the option once per file or list several files
        #[arg(short, long, value_name = "FILE", num_args = 1.., required = true)]
        input_files: Vec<PathBuf>,
        /// Directory for the output file if no output file or path is given (rust and cpp)
        #[arg(long, short = 'p')]
        input_base_path: Option<PathBuf>,

        /// Language to generate code for
        #[arg(short, long)]
        lang: Language,
        /// Output file or directory. rust/cpp: file, or directory to write <input name>.rs/.h
        /// (messages.rs/.h for multiple input files) into; python: directory of the maturin project
        #[arg(short, long)]
        output_file_or_path: Option<PathBuf>,

        /// Name (path relative to the generated header) of the bitis header lib that is included by
        /// and written next to the generated header (cpp only, default: bitis_lib.h)
        #[clap(long)]
        bitis_header_lib_file_name: Option<String>,

        /// Do not write the bitis header lib (cpp only)
        #[clap(long)]
        prevent_write_bitis_header_lib: bool,

        /// Do not set the bitis dependency in Cargo.toml of the maturin project (python only)
        #[clap(long)]
        prevent_update_bitis_lib_in_crate: bool
    },
    /// Setup directory and file structures
    Setup {
        /// system to set up
        #[arg(short, long)]
        system: System,
        output_path: PathBuf,
    }
}

/// Prints errors in the form `file:line:col: error: message` to stderr and exits with code 1.
fn exit_with_errors(errors: &[CompileError], files: &[SourceFile]) -> ! {
    for e in errors {
        eprintln!("{}", e.render(files));
    }
    std::process::exit(1);
}

fn write_output(path: &Path, content: &str) {
    if let Err(e) = fs::write(path, content) {
        print_error(format!("could not write file '{}': {}", path.display(), e));
    }
}

/// Output file of the rust and cpp targets.
fn output_file_for(output_file_opt: Option<PathBuf>, input_base_path: Option<PathBuf>, input_file_dir: Option<PathBuf>,
                   file_name: &str) -> PathBuf {
    if let Some(of) = output_file_opt {
        if of.is_dir() { of.join(file_name) } else { of }
    } else if let Some(dir) = input_base_path {
        dir.join(file_name)
    } else if let Some(dir) = input_file_dir {
        dir.join(file_name)
    } else {
        print_error("an output file or path (-o) is required if more than one input file is given".into());
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.debug > 0 { println!("BitisVersion: {}", crate_version!()); }

    match cli.command {
        Commands::Compile {
            input_files, input_base_path,
            lang, output_file_or_path: output_file_opt,
            bitis_header_lib_file_name, prevent_write_bitis_header_lib,
            prevent_update_bitis_lib_in_crate
        } => {
            if let Some(base) = &input_base_path {
                if !base.is_dir() { print_error(format!("base input path '{}' is not a directory", base.display())); }
                if cli.debug > 0 { println!("Base input path: {:?}", base); }
            }

            // ***
            let mut sources = Vec::new();
            for input_file in &input_files {
                if input_file.extension().and_then(|e| e.to_str()) != Some("bitis") {
                    print_error(format!("input file '{}' must have the extension '.bitis'", input_file.display()));
                }
                if cli.debug > 0 { println!("Reading input file: {:?}", input_file); }
                let content = match fs::read_to_string(input_file) {
                    Ok(c) => c,
                    Err(e) => print_error(format!("could not read input file '{}': {}", input_file.display(), e)),
                };
                sources.push(SourceFile { name: input_file.display().to_string(), content });
            }

            let processed_bitis = match compile_sources(&sources) {
                Ok(p) => p,
                Err(errors) => exit_with_errors(&errors, &sources),
            };
            if cli.debug > 2 { println!("processed_bitis: {:?}", processed_bitis); }
            if cli.debug > 0 {
                let summary = fixed_point_summary(&processed_bitis);
                if !summary.is_empty() { println!("FixedPoint summary:\n{}", summary); }
                println!("Schema hash: {}", schema_hash_hex(&processed_bitis));
            }
            if cli.debug > 1 { println!("Canonical schema:\n{}", canonical_schema(&processed_bitis)); }

            let (output_stem, input_file_dir) = if input_files.len() == 1 {
                let stem = input_files[0].file_stem().and_then(|s| s.to_str()).unwrap_or("messages").to_string();
                let dir = input_files[0].parent().map(|p| if p.as_os_str().is_empty() { PathBuf::from(".") } else { p.to_path_buf() });
                (stem, dir)
            } else { ("messages".to_string(), None) };

            match lang {
                Language::Rust => {
                    if cli.debug > 0 { println!("* Generating rust code ..."); }
                    let output_file = output_file_for(output_file_opt, input_base_path, input_file_dir,
                                                      &format!("{output_stem}.rs"));
                    if cli.debug > 0 { println!("output-file: '{}'", output_file.display()); }

                    let rendered = render_rust(&processed_bitis).unwrap_or_else(|e| exit_with_errors(&[e], &sources));
                    if cli.debug > 3 { println!("{}", rendered); }
                    write_output(&output_file, &rendered);
                    if cli.debug > 0 { println!("* Wrote file: {}", output_file.display()); }
                },
                Language::Python => {
                    if cli.debug > 0 { println!("* Generating python code ..."); }
                    let Some(output_path) = output_file_opt else {
                        print_error("an output path (-o) is required for the python target".into());
                    };
                    let output_path = match fs::canonicalize(&output_path) {
                        Ok(v) => v,
                        Err(e) => print_error(format!("output path '{}' could not be resolved: {}", output_path.display(), e)),
                    };
                    if cli.debug > 0 { println!("output-path: '{}'", output_path.display()); }
                    if !output_path.is_dir() {
                        print_error(format!("output path '{}' has to be a directory for the python target", output_path.display()));
                    }
                    let Some(lib_name) = output_path.file_name().and_then(|v| v.to_str()).map(|s| s.to_string()) else {
                        print_error("the output path has to end with the lib name".into());
                    };
                    if cli.debug > 0 { println!("* Lib-name: {}\n", lib_name); }

                    if !output_path.join(&lib_name).is_dir() {
                        print_error(format!("the python lib seems not to be set up correctly: expected subdir '{}' in output path ({}); \
                            there has to be a subdir with the same name as the selected directory", lib_name, output_path.display()));
                    }
                    if !output_path.join("src").is_dir() {
                        print_error(format!("the python lib seems not to be set up correctly: expected subdir 'src' in output path ({})",
                                            output_path.display()));
                    }

                    let out = render_python(&processed_bitis, &lib_name).unwrap_or_else(|e| exit_with_errors(&[e], &sources));
                    for (file, content) in [
                        ("src/messages.rs".to_string(), &out.messages_rs), ("src/pyrust.rs".to_string(), &out.pyrust_rs),
                        ("src/lib.rs".to_string(), &out.lib_rs), (format!("{lib_name}/{lib_name}.pyi"), &out.pyi),
                    ] {
                        let path = output_path.join(file);
                        write_output(&path, content);
                        if cli.debug > 0 { println!("Wrote file '{}'", path.display()); }
                    }

                    // *************************************
                    let toml_file = output_path.join("Cargo.toml");
                    if prevent_update_bitis_lib_in_crate {
                        if cli.debug > 0 { println!("- Not updating the bitis dependency in Cargo.toml"); }
                    }
                    else if toml_file.exists() {
                        if cli.debug > 0 { println!("- Found toml file and setting bitis-dependency"); }

                        let cct_content = match fs::read_to_string(&toml_file) {
                            Ok(content) => content,
                            Err(e) => print_error(format!("could not read '{}': {}", toml_file.display(), e)),
                        };
                        let mut toml_doc = match cct_content.parse::<DocumentMut>() {
                            Ok(v) => v,
                            Err(e) => print_error(format!("could not parse toml file '{}': {}", toml_file.display(), e)),
                        };
                        toml_doc["dependencies"]["bitis"] = value(crate_version!());
                        write_output(&toml_file, &toml_doc.to_string());
                    }
                    else { print_warn("There was no Cargo.toml found in the output directory.".into()) }

                    // ********************************************
                    let r = match Command::new("maturin").args(["develop"]).current_dir(&output_path)
                        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                        Ok(v) => v,
                        Err(e) => print_error(format!("could not execute 'maturin develop': {e}")),
                    };
                    let out = match r.wait_with_output() {
                        Ok(v) => v,
                        Err(e) => print_error(format!("error waiting for 'maturin develop': {e}")),
                    };
                    if !out.status.success() {
                        print_error(format!("'maturin develop' failed: {}", String::from_utf8_lossy(&out.stderr)));
                    }
                    println!("\n🎉 * Bitis compile and python lib build was successfully executed!\n");
                }
                Language::Cpp => {
                    if cli.debug > 0 { println!("* Generating cpp code ..."); }

                    let output_file = output_file_for(output_file_opt, input_base_path, input_file_dir,
                                                      &format!("{output_stem}.h"));
                    if cli.debug > 0 { println!("output-file: '{}'", output_file.display()); }

                    let bitis_header_lib_file_name = bitis_header_lib_file_name.unwrap_or("bitis_lib.h".to_string());
                    let stamp_name = stamp_name_from(output_file.file_stem().and_then(|s| s.to_str()).unwrap_or("messages"));

                    if cli.debug > 1 {
                        println!("Declaration order: {:?}", dependencies_process(JinjaData::for_cpp(&processed_bitis)));
                    }
                    let rendered = render_cpp(&processed_bitis, &bitis_header_lib_file_name, &stamp_name)
                        .unwrap_or_else(|e| exit_with_errors(&[e], &sources));
                    if cli.debug > 3 { println!("{}", rendered); }
                    write_output(&output_file, &rendered);
                    if cli.debug > 0 { println!("Written to {}", output_file.display()); }

                    if !prevent_write_bitis_header_lib {
                        let header_file_content = include_str!("../cpp_lib/bitis_lib.h");
                        let header_file_content = format!("//\n\
                            // Created by bitis_compiler {}\n\
                            //\n//\n\
                            // !!! THIS FILE IS AUTOMATICALLY GENERATED AND WILL BE OVERWRITTEN \n\
                            //     UNLESS THIS IS DISABLED BY BITIS COMPILER FLAGS !!!\n\
                            //     (please consult the documentation)\n//\n\n\
                            #pragma once\n\n\
                            //#define BITIS_CPP_LIB_VERSION \"{}\"\n\n{}", crate_version!(), crate_version!(),
                        header_file_content);

                        // the header lib is included relative to the generated header
                        let output_dir = output_file.parent().filter(|p| !p.as_os_str().is_empty())
                            .map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
                        let header_file_name = output_dir.join(&bitis_header_lib_file_name);
                        let is_symlink = fs::symlink_metadata(&header_file_name).map(|m| m.is_symlink()).unwrap_or(false);
                        if !is_symlink {
                            write_output(&header_file_name, &header_file_content);
                            if cli.debug > 0 { println!("Written cpp-header-lib to {}", header_file_name.display()); }
                        }
                        else if cli.debug > 0 {
                            println!("Not writing cpp-header-lib to {} as it is a symlink", header_file_name.display());
                        }
                    }
                }
        } },
        Commands::Setup{system, output_path} => {
            match system {
                System::Maturin => {
                    let Some(lib_name) = output_path.file_name().and_then(|v| v.to_str()).map(|s| s.to_string()) else {
                        print_error("the output path has to be the future lib name".into());
                    };
                    if cli.debug > 1 { println!("* Lib-name: {}\n", lib_name); }

                    // check if venv is enabled
                    let r = match Command::new("pip").args(["-V"]).output() {
                        Ok(v) => v, Err(_) => print_error("could not find pip executable".into()),
                    };
                    if !String::from_utf8_lossy(&r.stdout).contains(".venv") {
                        print_error("a venv needs to be activated for setting up the python lib".into());
                    }
                    // check for maturin
                    if Command::new("maturin").args(["--version"]).output().is_err() {
                        print_error("maturin python package not installed. Please install it with\n\n  pip install maturin\n".into());
                    }
                    // make dir
                    if !output_path.exists() && fs::create_dir_all(&output_path).is_err() {
                        print_error(format!("unable to create output directory {}", output_path.display()));
                    }
                    // if there is no src directory, call maturin init
                    if !output_path.join("src").exists() {
                        if cli.debug > 1 { println!("Initializing maturin project ..."); }
                        let r = match Command::new("maturin").args(["init", "-b", "pyo3"]).current_dir(&output_path).output() {
                            Ok(v) => v, Err(e) => print_error(format!("'maturin init' failed: {}", e)),
                        };
                        if !r.status.success() {
                            print_error(format!("'maturin init' failed: {}", String::from_utf8_lossy(&r.stderr)));
                        }
                        if cli.debug > 1 { println!("maturin response: {:?}\n  done!", r); }
                    }
                    let py_code_dir = output_path.join(&lib_name);
                    if !py_code_dir.exists() {
                        if fs::create_dir_all(&py_code_dir).is_err() {
                            print_error(format!("could not create directory '{}'", py_code_dir.display()));
                        }
                        write_output(&py_code_dir.join("__init__.py"), &format!("from .{lib_name} import *"));
                        write_output(&py_code_dir.join("py.typed"), "");
                    }
                    println!("*** Project successfully setup.");
                }
            }
        }
    }
}
