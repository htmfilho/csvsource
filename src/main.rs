use clap::{Arg, ArgMatches, App, ErrorKind};

use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::io::prelude::*;
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
        .arg(Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .default_value("comma")
                .value_name("comma | semicolon | tab")
                .help("The supported CSV delimiter used in the file."))
        .arg(Arg::new("table")
                .long("table")
                .short('t')
                .value_name("database_table_name")
                .help("Database table name if it is different from the name of the CSV file."))
        .arg(Arg::new("headers")
                .long("headers")
                .short('h')
                .default_value("true")
                .value_name("true | false")
                .help("Consider the first line in the file as headers to columns. They are also used as sql column names unless specified otherwise."))
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
        .arg(Arg::new("prefix")
                .long("prefix")
                .short('p')
                .value_name("file")
                .help("File with the content to prefix the sql file. Example: it can be used to create the target table."))
        .arg(Arg::new("suffix")
                .long("suffix")
                .short('s')
                .value_name("file")
                .help("File with the content to suffix the sql file. Example: it can be used to create indexes."))
        .get_matches();

    let args = load_arguments(matches);

    match process_csv(args) {
        Ok(())   => println!("CSV file processed successfully!"),
        Err(err) => println!("Error: {}.", err)
    };
}

fn process_csv(args: Arguments) -> Result<(), io::Error> {
    if !Path::new(args.csv.as_str()).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "CSV file not found"));
    }

    let csv_file = File::open(args.csv.clone())?;
    let reader = io::BufReader::new(csv_file);
    let csv_reader = csv::ReaderBuilder::new()
                .has_headers(args.has_headers)
                .from_reader(reader);

    return generate_sql_file(args, csv_reader);
}

fn generate_sql_file(args: Arguments, csv_reader: csv::Reader<io::BufReader<File>>) -> Result<(), io::Error> {
    let sql_file = File::create(get_file_name_without_extension(&args.csv) + ".sql").expect("Unable to create file");
    let mut writer = BufWriter::new(sql_file);

    if let Err(err) = append_file_content(args.prefix.clone(), &mut writer) {
        return Err(err);
    }

    if let Err(err) = generate_sql(&args, csv_reader, &mut writer) {
        return Err(err);
    }

    if let Err(err) = append_file_content(args.suffix, &mut writer) {
        return Err(err);
    }

    return Ok(());
}

fn generate_sql(args: &Arguments, mut csv_reader: csv::Reader<io::BufReader<File>>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    let insert_fields =
        if args.columns.is_empty() && args.has_headers {
            get_insert_fields(csv_reader.headers()?)
        } else {
            args.get_insert_fields()
        };

    let mut chunk_count = 0;
    let mut chunk_insert_count = 0;
    let mut insert_separator = ";";

    write!(writer, "begin transaction")?;

    for record in csv_reader.records() {
        if chunk_insert_count == 0 {
            if args.chunk > 0 && chunk_count == args.chunk {
                write!(writer, ";\n\ncommit;\n\nbegin transaction")?;
                chunk_count = 0;
            }

            write!(writer, "{}\n\ninsert into {} {} values", insert_separator, args.table.as_str(), insert_fields)?;
            insert_separator = "";
            chunk_count += 1;
        }

        match record {
            Ok(row) => write!(writer, "{}\n{}", insert_separator, get_values(&row))?,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }

        if args.chunk_insert > 0 {
            chunk_insert_count += 1;
            insert_separator = ",";
            if args.chunk_insert == chunk_insert_count {
                chunk_insert_count = 0;
                insert_separator = ";";
            }
        } else {
            insert_separator = ";";
        }
    }

    writeln!(writer, ";\n\ncommit;")?;

    return Ok(());
}

fn append_file_content(path: String, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    if !Path::new(path.as_str()).exists() {
        return Ok(());
    }
    
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        writeln!(writer, "{}", line?)?;
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
        values.push_str(&get_value(result));
        separator = ", "
    }
    values.push_str(")");
    return values;
}

fn get_value(result: &str) -> String {
    let mut value = String::new();
    if is_number(String::from(result)) {
        value.push_str(result);
    } else if is_boolean(String::from(result)) {
        value.push_str(result);
    } else {
        if result.is_empty() {
            value.push_str("NULL");
        } else {
            value.push_str("'");
            value.push_str(&result.replace("'", "''"));
            value.push_str("'");
        }
    }
    return value;
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

    return match test {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn is_boolean(str: String) -> bool {
    let tr: String = String::from("true");
    let fs: String = String::from("false");

    return tr.eq(&str.to_lowercase()) || fs.eq(&str.to_lowercase());
}

struct Arguments {
    csv          : String,
    delimiter    : u8,
    has_headers  : bool,
    table        : String,
    columns      : Vec<String>,
    chunk        : usize,
    chunk_insert : usize,
    prefix       : String,
    suffix       : String,
}

impl Arguments {
    fn get_insert_fields(&self) -> String {
        let mut insert_fields = String::from("(");
        let mut separator = "";
        for column in &self.columns {
            insert_fields.push_str(separator);
            insert_fields.push_str(column.as_str());
            separator = ", "
        }
        insert_fields.push_str(")");
        return insert_fields;
    }
}

fn load_arguments(matches: ArgMatches) -> Arguments {
    let mut arguments = Arguments{
        csv: String::from(""),
        delimiter: b',',
        has_headers: true,
        table: String::from(""),
        columns: Vec::new(),
        chunk: 0,
        chunk_insert: 0,
        prefix: String::from(""),
        suffix: String::from(""),
    };

    if let Some(csv) = matches.value_of("csv") {
        arguments.csv = String::from(csv);
    }

    if let Some(delimiter) = matches.value_of("delimiter") {
        match delimiter {
            "comma"     => arguments.delimiter = b',',
            "semicolon" => arguments.delimiter = b';',
            "tab"       => arguments.delimiter = b'\t',
            _ => App::new("Roma").error(ErrorKind::InvalidValue, "Invalid delimiter. Use 'comma', 'semicolon', or 'tab'.").exit()
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

    if let Some(prefix) = matches.value_of("prefix") {
        arguments.prefix = String::from(prefix);
    }

    if let Some(suffix) = matches.value_of("suffix") {
        arguments.suffix = String::from(suffix);
    }
    
    return arguments;
}

fn get_file_name_without_extension(csv_file_name: &String) -> String {
    let last_dot_pos = csv_file_name.rfind('.');
    let last_slash_pos = csv_file_name.rfind('/');
    return match last_dot_pos {
        Some(pos_dot) => {
            match last_slash_pos {
                Some(pos_slash) => csv_file_name[(pos_slash + 1)..pos_dot].to_string(),
                None => csv_file_name[..pos_dot].to_string(),
            }
        },
        None => csv_file_name.to_string(),
    }
}