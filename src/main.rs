use clap::{Arg, ArgMatches, App, ErrorKind};
use std::io;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;
use std::str::ParseBoolError;

fn main() {
    let matches = App::new("Roma")
        .version("0.1.0")
        .author("Hildeberto Mendonca <me@hildeberto.com>")
        .about("Converts a CSV file to SQL Insert Statements.")
        .arg(Arg::new("csv")
                .long("csv")
                .short('f')
                .value_name("file")
                .required(true)
                .takes_value(true)
                .help("Relative or absolute path to the CSV file. The name of the file is also used was table name unless specified otherwise."))
        .arg(Arg::new("separator")
                .long("separator")
                .short('s')
                .default_value("comma")
                .value_name("comma | tab")
                .help("The supported CSV separator used in the file."))
        .arg(Arg::new("labels")
                .long("labels")
                .short('l')
                .default_value("true")
                .value_name("true | false")
                .help("Consider the first line in the file as labels to columns. They are also used as sql column names unless specified otherwise."))
        .arg(Arg::new("table")
                .long("table")
                .short('t')
                .value_name("database_table_name")
                .help("Database table name if it is different from the name of the CSV file."))
        .arg(Arg::new("columns")
                .long("column")
                .short('c')
                .required_if_eq("labels", "false")
                .multiple_occurrences(true)
                .value_name("database_column_names")
                .help("Columns of the database table if different from the name of the labels."))
        .arg(Arg::new("chunk")
                .long("chunk")
                .short('k')
                .default_value("0")
                .value_name("#")
                .help("Size of the transaction chunk, indicating how many insert statements are put within a transaction scope."))
        .arg(Arg::new("chunk_insert")
                .long("chunkinsert")
                .short('i')
                .default_value("0")
                .value_name("#")
                .help("Size of the insert chunk, indicating how many lines of the CSV files will be put in a single insert statement."))
        .get_matches();

    let args = load_arguments(matches);

    //println!("csv: {}", args.csv);
    //println!("separator: {}", args.separator);
    //println!("has_labels: {}", args.has_labels);
    //println!("table: {}", args.table);
    //println!("columns: {:#?}", args.columns);
    //println!("chunk: {}", args.chunk);
    //println!("insert chunk: {}", args.chunk_insert);

    match process_csv(args) {
        Ok(()) => println!("CSV file processed successfully!"),
        Err(err) => println!("Error: {}.", err)
    };
}

fn process_csv(args: Arguments) -> Result<(), io::Error> {
    if !Path::new(args.csv.as_str()).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "CSV file not found"));
    }
    return Ok(());
}

struct Arguments {
    csv          : String,
    separator    : String,
    has_labels   : bool,
    table        : String,
    columns      : Vec<String>,
    chunk        : usize,
    chunk_insert : usize
}

fn load_arguments(matches: ArgMatches) -> Arguments {
    let mut arguments = Arguments{
        csv: String::from(""),
        separator: String::from(""),
        has_labels: true,
        table: String::from(""),
        columns: Vec::new(),
        chunk: 0,
        chunk_insert: 0,
    };

    if let Some(csv) = matches.value_of("csv") {
        arguments.csv = String::from(csv);
    }

    if let Some(separator) = matches.value_of("separator") {
        match separator {
            "comma" => arguments.separator = String::from(separator),
            "tab"   => arguments.separator = String::from(separator),
            _ => App::new("Roma").error(ErrorKind::InvalidValue, "Invalid separator. Use 'comma' or 'tab'.").exit()
        }
    }

    if let Some(labels) = matches.value_of("labels") {
        let has_labels: Result<bool, ParseBoolError> = FromStr::from_str(labels);
        arguments.has_labels = has_labels.ok().unwrap();
    }

    if let Some(table) = matches.value_of("table") {
        arguments.table = String::from(table);
    }

    if let Some(cols) = matches.values_of("columns") {
        let columns: Vec<&str> = cols.collect();
        let mut columns_vec: Vec<String> = Vec::new();
        for s in &columns {
            columns_vec.push(s.to_string());
        }
        arguments.columns = columns_vec;
    }
    
    if let Some(chunk) = matches.value_of("chunk") {
        arguments.chunk = String::from(chunk).parse::<usize>().unwrap();
    }

    if let Some(insert_chunk) = matches.value_of("chunk_insert") {
        arguments.chunk_insert = String::from(insert_chunk).parse::<usize>().unwrap();
    }
    
    return arguments;
}