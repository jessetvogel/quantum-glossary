use std::{
    fs::{self, create_dir_all, File},
    io::{Result, Write},
    path::PathBuf,
};

use colored::Colorize;
use parser::Parser;

mod lexer;
mod parser;

const SOURCE_DIRECTORY: &str = "../tex";
const TARGET_DIRECTORY: &str = "../public/data";
const TOPICS_JS: &str = "../public/data/topics.js";
const EXAMPLES_JS: &str = "../public/data/examples.js";
const CATEGORIES: &[(&str, &str)] = &[("ALG", "Algorithms"), ("GEN", "General")];

fn main() {
    // Step 0: Make sure TARGET_DIRECTORY exists, and topics and examples subdirectories
    for path in &[
        TARGET_DIRECTORY,
        &format!("{TARGET_DIRECTORY}/topics"),
        &format!("{TARGET_DIRECTORY}/examples"),
    ] {
        if let Err(err) = create_dir_all(path) {
            println!(
                "{}: Failed to create target directory '{path}' ({err})",
                "Error".red()
            );
            std::process::exit(1);
        }
    }

    // Step 1: Find all .tex files
    let tex_files = find_tex_files();
    println!(
        "Found {} .tex files in {}",
        tex_files.len(),
        SOURCE_DIRECTORY
    );

    // Step 2: Parse all .tex files
    let mut parser = Parser::new(TARGET_DIRECTORY);
    for (prefix, path) in &tex_files {
        // Open file
        let file = match File::open(path) {
            Err(err) => {
                println!("{}: failed to open file '{path:?}' ({err})", "Error".red());
                return;
            }
            Ok(file) => file,
        };
        // Parse file
        print!("Parsing {} .. ", path.to_str().unwrap());
        std::io::stdout().flush().unwrap();
        match parser.parse(prefix, file) {
            Ok(()) => {
                println!("{}", "ok".green());
            }
            Err(error) => {
                println!("{}", "failed".red());
                println!("\n{}", error.message);
                return;
            }
        }
    }

    // Step 3: Write `topics.js`
    match write_topics_js(&parser) {
        Ok(()) => {}
        Err(err) => {
            println!(
                "{}: Failed to write to {} ({err})",
                "Error".red(),
                TOPICS_JS
            );
            return;
        }
    }

    // Step 4: Write `examples.js`
    match write_examples_js(&parser) {
        Ok(()) => {}
        Err(err) => {
            println!(
                "{}: Failed to write to {} ({err})",
                "Error".red(),
                TOPICS_JS
            );
            return;
        }
    }

    // Done
    println!("\nDone! 🎉 ({} topics)", parser.topics().len());
}

fn find_tex_files() -> Vec<(&'static str, PathBuf)> {
    let mut tex_files = Vec::new();
    for &(prefix, category) in CATEGORIES {
        let path = format!("{SOURCE_DIRECTORY}/{category}");
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .map_or(false, |s| s.ends_with(".tex") && !s.starts_with("--"))
            {
                tex_files.push((prefix, entry.path()));
            }
        }
    }
    tex_files
}

fn write_topics_js(parser: &Parser) -> Result<()> {
    let mut topics_js = File::create(TOPICS_JS)?;
    topics_js.write(b"const topics = {\n")?;
    for (uid, name) in parser.topics() {
        topics_js.write(format!("  \"{}\": \"{}\",\n", uid, name).as_bytes())?;
    }
    topics_js.write(b"}")?;
    Ok(())
}

fn write_examples_js(parser: &Parser) -> Result<()> {
    let mut examples_js = File::create(EXAMPLES_JS)?;
    examples_js.write(b"const examples = [\n")?;
    for uid in parser.examples() {
        examples_js.write(format!("  \"{}\",\n", uid).as_bytes())?;
    }
    examples_js.write(b"]")?;
    Ok(())
}
