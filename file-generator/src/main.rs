mod db;

use clap::Parser;
use serde_json::{json, Value};
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

    /// Path to the output file
    #[arg(short, long)]
    output: PathBuf,

    // Data source arguments
    /// Path to a JSON data file. Mutually exclusive with database arguments.
    #[arg(short, long, conflicts_with_all = &["connection_string", "query"])]
    data: Option<PathBuf>,

    /// The database connection string. Requires --query.
    #[arg(long, requires = "query")]
    connection_string: Option<String>,

    /// The SQL query to execute. Requires --connection-string.
    #[arg(long, requires = "connection_string")]
    query: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Determine the data source and fetch data
    let template_data: Value = if let Some(data_path) = args.data {
        // File-based data source
        let data_str = fs::read_to_string(data_path)?;
        serde_json::from_str(&data_str)?
    } else if let (Some(conn_str), Some(query)) = (args.connection_string, args.query) {
        // Database data source
        let db_results = db::fetch_data(&conn_str, &query).await?;
        json!({ "data": db_results })
    } else {
        // This case should be prevented by clap's argument validation,
        // but we handle it just in case by providing an empty JSON object.
        // A more robust solution might return an error.
        json!({})
    };

    // Read template file
    let template_str = fs::read_to_string(&args.template)?;

    // Create Tera context
    let context = Context::from_value(template_data)?;

    // Render template
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template_str)?;
    let rendered = tera.render("template", &context)?;

    // Write output file
    fs::write(&args.output, rendered)?;

    println!("File generated successfully at {:?}", args.output);

    Ok(())
}