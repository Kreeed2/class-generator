use clap::Parser;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

/// A command-line tool to generate files from templates and data.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the template file
    #[arg(short, long)]
    template: PathBuf,

    /// Path to the JSON data file
    #[arg(short, long)]
    data: PathBuf,

    /// Path to the output file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read template file
    let template_str = fs::read_to_string(&args.template)?;

    // Read data file
    let data_str = fs::read_to_string(&args.data)?;
    let data: Value = serde_json::from_str(&data_str)?;

    // Create Tera context
    let context = Context::from_value(data)?;

    // Render template
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template_str)?;
    let rendered = tera.render("template", &context)?;

    // Write output file
    fs::write(&args.output, rendered)?;

    println!("File generated successfully at {:?}", args.output);

    Ok(())
}
