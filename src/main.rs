use clap::{Arg, ArgMatches, App, ErrorKind};
use itertools::intersperse;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;
use std::str::ParseBoolError;
use tinytemplate::TinyTemplate;

fn main() {
    let matches = App::new("Roma")
        .version("0.6.0")
        .author("Hildeberto Mendonca <me@hildeberto.com>")
        .about("Converts a CSV file to SQL Insert Statements.")
        .arg(Arg::new("csv")
            .long("csv")
            .short('f')
            .value_name("file")
            .required(true)
            .takes_value(true)
            .help("Relative or absolute path to the CSV file. The file's name is also used as table name and sql file's name, unless specified otherwise by the arguments `--table` and `--sql` respectivelly."))
        .arg(Arg::new("sql")
            .long("sql")
            .short('q')
            .value_name("file")
            .help("Relative or absolute path to the SQL file."))
        .arg(Arg::new("delimiter")
            .long("delimiter")
            .short('d')
            .default_value("comma")
            .value_name("comma | semicolon | tab")
            .help("The supported CSV value delimiter used in the file."))
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
            .help("Size of the insert chunk, indicating how many lines of the CSV files are put in a single insert statement."))
        .arg(Arg::new("prefix")
            .long("prefix")
            .short('p')
            .value_name("file")
            .help("File with the content to put at the beginning of the SQL file. Example: It can be used to create the target table."))
        .arg(Arg::new("suffix")
            .long("suffix")
            .short('s')
            .value_name("file")
            .help("File with the content to put at the end of the SQL file. Example: It can be used to create indexes."))
        .arg(Arg::new("with_transaction")
            .long("with_transaction")
            .short('w')
            .default_value("false")
            .value_name("true | false")
            .help("Indicates whether SQL statements are put in a transaction block or not. This argument is ignored if the argument chunk is used."))
        .arg(Arg::new("typed")
            .long("typed")
            .short('y')
            .default_value("false")
            .value_name("true | false")
            .help("Indicates whether the values type are declared, automatically detected or everything is taken as string."))
        .get_matches();

    let args = Arguments::new_from_console(matches);

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

    generate_sql_file(args, csv_reader)
}

fn generate_sql_file(args: Arguments, csv_reader: csv::Reader<io::BufReader<File>>) -> Result<(), io::Error> {
    let sql_file = File::create(&args.sql).expect("Unable to create sql file");
    let mut writer = BufWriter::new(sql_file);

    let context = &TemplateContext {
        table: args.table.to_string()
    };
    append_file_content(args.prefix.clone(), context, &mut writer)?;
    generate_sql(&args, csv_reader, &mut writer)?;
    append_file_content(args.suffix, context, &mut writer)?;

    Ok(())
}

fn generate_sql(args: &Arguments, mut csv_reader: csv::Reader<io::BufReader<File>>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    let insert_fields = args.format_fields(args.get_fields(csv_reader.headers()?));

    let mut chunk_count = 0;
    let mut chunk_insert_count = 0;
    let mut insert_separator = ";\n\n";

    if args.with_transaction {
        write!(writer, "begin transaction")?;
    } else {
        insert_separator = "";
    }

    for record in csv_reader.records() {
        if chunk_insert_count == 0 {
            if args.chunk > 0 && chunk_count == args.chunk {
                write!(writer, ";\n\ncommit;\n\nbegin transaction")?;
                chunk_count = 0;
            }

            write!(writer, "{}insert into {} {} values", insert_separator, args.table.as_str(), insert_fields)?;
            insert_separator = "";
            chunk_count += 1;
        }

        match record {
            Ok(row) => write!(writer, "{}\n{}", insert_separator, get_values(args, &row))?,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }

        if args.chunk_insert > 0 {
            chunk_insert_count += 1;
            insert_separator = ",";
            if args.chunk_insert == chunk_insert_count {
                chunk_insert_count = 0;
                insert_separator = ";\n\n";
            }
        } else {
            insert_separator = ";\n\n";
        }
    }

    if args.with_transaction {
        write!(writer, ";\n\ncommit;")?
    } else {
        write!(writer, ";")?
    }

    Ok(())
}

#[derive(Serialize)]
struct TemplateContext {
    table: String,
}

fn append_file_content(path: String, context: &TemplateContext, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
    if !Path::new(path.as_str()).exists() {
        return Ok(());
    }
    
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut template = String::new();

    for line in reader.lines() {
        template.push_str(line.unwrap().as_str());
        template.push_str("\n");
    }

    let mut tt = TinyTemplate::new();
    let rendered = match tt.add_template("append", template.as_str()) {
        Ok(..) => match tt.render("append", context) {
            Ok(r) => r,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e))
        },
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e))
    };

    writeln!(writer, "{}", rendered)?;

    Ok(())
}

fn get_values(args: &Arguments, record: &csv::StringRecord) -> String {
    let mut values = String::new();
    let mut separator = "";

    for result in record {
        values.push_str(separator);
        if args.typed {
            values.push_str(&get_value(result));
        } else {
            values.push_str("'");
            values.push_str(&result.replace("'", "''"));
            values.push_str("'");
        }
        separator = ", "
    }

    format!("({})", values)
}

fn get_value(result: &str) -> String {
    let mut value = String::new();

    if is_number(result) {
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

    value
}

fn is_number(str: &str) -> bool {
    if str.is_empty() {
        return false;
    }

    let test = str.parse::<f64>();

    return match test {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn is_boolean(str: String) -> bool {
    let tr = "true";
    let fs = "false";

    return tr.eq(&str.to_lowercase()) || fs.eq(&str.to_lowercase());
}

struct Arguments {
    csv              : String,
    sql              : String,
    delimiter        : u8,
    has_headers      : bool,
    table            : String,
    columns          : Vec<String>,
    chunk            : usize,
    chunk_insert     : usize,
    prefix           : String,
    suffix           : String,
    with_transaction : bool,
    typed            : bool,
}

impl Arguments {
    pub fn new_from_console(matches: ArgMatches) -> Self {
        let mut arguments = Self {
            csv: String::from(""),
            sql: String::from(""),
            delimiter: b',',
            has_headers: true,
            table: String::from(""),
            columns: Vec::new(),
            chunk: 0,
            chunk_insert: 0,
            prefix: String::from(""),
            suffix: String::from(""),
            with_transaction: false,
            typed: false,
        };

        if let Some(csv) = matches.value_of("csv") {
            arguments.csv = String::from(csv);
        }

        let sql = matches.value_of("sql");
        match sql {
            Some(q) => arguments.sql = String::from(q),
            None => arguments.sql = get_file_name_without_extension(&arguments.csv) + ".sql",
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
            if arguments.chunk > 0 {
                arguments.with_transaction = true;
            }
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

        if let Some(with_transaction) = matches.value_of("with_transaction") {
            if arguments.chunk <= 0 {
                let result: Result<bool, ParseBoolError> = FromStr::from_str(with_transaction);
                arguments.with_transaction = result.ok().unwrap();
            }
        }

        if let Some(typed) = matches.value_of("typed") {
            let result: Result<bool, ParseBoolError> = FromStr::from_str(typed);
            arguments.typed = result.ok().unwrap();
        }

        return arguments;
    }

    fn get_fields(&self, headers: &csv::StringRecord) -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        if self.columns.is_empty() && self.has_headers {
            for header in headers {
                fields.push(header.to_string());
            }
        } else {
            for column in &self.columns {
                fields.push(column.to_string());
            }
        }
        fields
    }

    fn format_fields(&self, fields: Vec<String>) -> String {
        let insert_fields: String = intersperse(fields, ", ".to_string()).collect();
        format!("({})", insert_fields)
    }
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