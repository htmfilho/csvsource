use clap::{Arg, ArgMatches, App};
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
                .multiple_occurrences(true)
                .value_name("database_column_names")
                .help("Columns of the database table if different from the name of the labels."))
        .arg(Arg::new("chunk")
                .long("chunk")
                .short('k')
                .default_value("0")
                .value_name("#")
                .help("Size of the transaction chunk, indicating how many insert statements are put within a transaction scope."))
        .arg(Arg::new("insert_chunk")
                .long("insertchunk")
                .short('i')
                .default_value("0")
                .value_name("#")
                .help("Size of the insert chunk, indicating how many lines of the CSV files will be put in a single insert statement."))
        .get_matches();

    let arguments = load_arguments(matches);

    println!("csv: {}", arguments.csv);
    println!("separator: {}", arguments.separator);
    println!("has_labels: {}", arguments.has_labels);
    println!("table: {}", arguments.table);
    println!("columns: {:#?}", arguments.columns);
    println!("chunk: {}", arguments.chunk);
    println!("insert chunk: {}", arguments.insert_chunk);
}

struct Arguments {
    csv          : String,
    separator    : String,
    has_labels   : bool,
    table        : String,
    columns      : Vec<String>,
    chunk        : usize,
    insert_chunk : usize
}

fn load_arguments(matches: ArgMatches) -> Arguments {
    let mut arguments = Arguments{
        csv: String::from(""),
        separator: String::from(""),
        has_labels: true,
        table: String::from(""),
        columns: Vec::new(),
        chunk: 0,
        insert_chunk: 0,
    };

    if let Some(csv) = matches.value_of("csv") {
        arguments.csv = String::from(csv);
    }

    if let Some(separator) = matches.value_of("separator") {
        arguments.separator = String::from(separator);
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

    if let Some(insert_chunk) = matches.value_of("insert_chunk") {
        arguments.insert_chunk = String::from(insert_chunk).parse::<usize>().unwrap();
    }
    
    return arguments;
}