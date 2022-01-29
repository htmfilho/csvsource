use clap::{Arg, ArgMatches, App, ErrorKind};

use std::fs::File;
use std::io;
use std::path::Path;
use std::process;
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
        .arg(Arg::new("headers")
                .long("headers")
                .short('h')
                .default_value("true")
                .value_name("true | false")
                .help("Consider the first line in the file as headers to columns. They are also used as sql column names unless specified otherwise."))
        .arg(Arg::new("table")
                .long("table")
                .short('t')
                .value_name("database_table_name")
                .help("Database table name if it is different from the name of the CSV file."))
        .arg(Arg::new("columns")
                .long("column")
                .short('c')
                .required_if_eq("headers", "false")
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

    match process_csv(args) {
        Ok(()) => println!("CSV file processed successfully!"),
        Err(err) => println!("Error: {}.", err)
    };
}

fn process_csv(args: Arguments) -> Result<(), io::Error> {
    if !Path::new(args.csv.as_str()).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "CSV file not found"));
    }

    let f = File::open(args.csv)?;
    let reader = io::BufReader::new(f);
    let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(args.has_headers)
                .from_reader(reader);

    let insert_fields = get_insert_fields(csv_reader.headers()?);

    for result in csv_reader.records() {
        match result {
            Ok(record) => println!("\ninsert into {} {} values {};", args.table.as_str(), insert_fields, get_values(&record)),
            Err(err) => {
                println!("Error reading CSV from file: {}", err);
                process::exit(1);
            }
        }
    }

    return Ok(());
}

fn get_insert_fields(headers: &csv::StringRecord) -> String {
    let mut insert_fields = String::from("(");
    let mut separator = "";
    for result in headers {
        insert_fields.push_str(separator);
        insert_fields.push_str(result);
        separator = ", "
    }
    insert_fields.push_str(")");
    return insert_fields;
}

fn get_values(record: &csv::StringRecord) -> String {
    let mut values = String::from("(");
    let mut separator = "";
    for result in record {
        values.push_str(separator);
        if is_number(String::from(result)) {
            values.push_str(result);
        } else {
            if result.is_empty() {
                values.push_str("NULL");
            } else {
                values.push_str("'");
                values.push_str(result);
                values.push_str("'");
            }
        }
        separator = ", "
    }
    values.push_str(")");
    return values;
}

fn is_number(str: String) -> bool {
    if str.is_empty() {
        return false;
    }

    for c in str.chars() {
        if !c.is_numeric() {
            return is_decimal(str);
        }
    }
    return true;
}

fn is_decimal(str: String) -> bool {

    let test = str.parse::<f64>();

    match test {
        Ok(_ok) => return true,
        Err(_e) => return false, 
    }
}

struct Arguments {
    csv          : String,
    separator    : String,
    has_headers  : bool,
    table        : String,
    columns      : Vec<String>,
    chunk        : usize,
    chunk_insert : usize
}

fn load_arguments(matches: ArgMatches) -> Arguments {
    let mut arguments = Arguments{
        csv: String::from(""),
        separator: String::from(""),
        has_headers: true,
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

    if let Some(headers) = matches.value_of("headers") {
        let has_headers: Result<bool, ParseBoolError> = FromStr::from_str(headers);
        arguments.has_headers = has_headers.ok().unwrap();
    }

    let table = matches.value_of("table");
    match table {
        Some(tbl) => arguments.table = String::from(tbl),
        None => arguments.table = get_file_name_without_extension(&arguments.csv),
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

fn get_file_name_without_extension(csv_file_name: &String) -> String {
    let last_dot_pos = csv_file_name.rfind('.');
    let last_slash_pos = csv_file_name.rfind('/');
    match last_dot_pos {
        Some(pos_dot) => {
            match last_slash_pos {
                Some(pos_slash) => return csv_file_name[(pos_slash + 1)..pos_dot].to_string(),
                None => return csv_file_name[..pos_dot].to_string(),
            }
        },
        None => return csv_file_name.to_string(),
    }
}