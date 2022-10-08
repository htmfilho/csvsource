use serde::Serialize;
use itertools::intersperse;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::io::prelude::*;
use std::result::Result;
use tinytemplate::TinyTemplate;

pub fn convert_to_sql(args: Arguments) -> Result<(), io::Error> {
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

pub struct Arguments {
    pub csv              : String,
    pub sql              : String,
    pub delimiter        : u8,
    pub has_headers      : bool,
    pub table            : String,
    pub columns          : Vec<String>,
    pub chunk            : usize,
    pub chunk_insert     : usize,
    pub prefix           : String,
    pub suffix           : String,
    pub with_transaction : bool,
    pub typed            : bool,
}

impl Arguments {
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