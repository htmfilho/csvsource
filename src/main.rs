use clap::{Arg, ArgMatches, App, ErrorKind};
use std::str::FromStr;
use std::str::ParseBoolError;
use csvsource::Arguments;
use csvsource::target::Target;
use csvsource::target::csv::TargetCsv;
use csvsource::target::sql::TargetSql;

fn main() {
    let matches = App::new("CSVSource")
        .version("0.6.0")
        .author("Hildeberto Mendonca <me@hildeberto.com>")
        .about("Converts a CSV file to SQL Insert Statements.")
        .arg(Arg::new("source")
            .long("source")
            .short('f')
            .value_name("file")
            .required(true)
            .takes_value(true)
            .help("Relative or absolute path to the CSV file. The file's name is also used as table name and target file's name, unless specified otherwise by the arguments `--table` and `--target` respectivelly."))
        .arg(Arg::new("target")
            .long("target")
            .short('g')
            .value_name("file")
            .help("Relative or absolute path to the target file."))
        .arg(Arg::new("target_type")
            .long("target_type")
            .short('e')
            .default_value("sql")
            .value_name("sql | csv")
            .help("The type of output we want to generate from the source."))
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

    let args = arguments_from_console(matches);
    let target: Box<dyn Target> = match args.target_type.as_str() {
        "csv" => Box::new(TargetCsv{}),
        _     => Box::new(TargetSql{}),
    };

    match target.convert(args) {
        Ok(())   => println!("CSV file processed successfully!"),
        Err(err) => println!("Error: {}.", err)
    };
}

fn arguments_from_console(matches: ArgMatches) -> Arguments {
    let mut arguments = Arguments {
        source: String::from(""),
        target: String::from(""),
        target_type: String::from(""),
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

    if let Some(source) = matches.value_of("source") {
        arguments.source = String::from(source);
    }

    let target = matches.value_of("target");
    match target {
        Some(q) => arguments.target = String::from(q),
        None => arguments.target = get_file_name_without_extension(&arguments.source) + ".sql",
    }

    let target_type = matches.value_of("target_type");
    match target_type {
        Some(tt) => arguments.target_type = String::from(tt),
        None => arguments.target_type = String::from("sql")
    }

    if let Some(delimiter) = matches.value_of("delimiter") {
        match delimiter {
            "comma"     => arguments.delimiter = b',',
            "semicolon" => arguments.delimiter = b';',
            "tab"       => arguments.delimiter = b'\t',
            _ => App::new("CSVSource").error(ErrorKind::InvalidValue, "Invalid delimiter. Use 'comma', 'semicolon', or 'tab'.").exit()
        }
    }

    if let Some(headers) = matches.value_of("headers") {
        let has_headers: Result<bool, ParseBoolError> = FromStr::from_str(headers);
        arguments.has_headers = has_headers.ok().unwrap();
    }

    let table = matches.value_of("table");
    match table {
        Some(tbl) => arguments.table = String::from(tbl),
        None => arguments.table = get_file_name_without_extension(&arguments.source),
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
