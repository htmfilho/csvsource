pub struct Arguments {
    pub source           : String,
    pub target           : String,
    pub target_type      : String,
    pub delimiter        : u8,
    pub has_headers      : bool,
}

pub mod target {
    use std::io;
    use std::result::Result;
    
    use crate::Arguments;

    pub trait Target {
        fn convert(&self, args: Arguments) -> Result<(), io::Error>;
    }

    pub mod sql {
        use serde::Serialize;
        use std::io;
        use std::fs::File;
        use std::io::{BufWriter, Write};
        use std::path::Path;
        use std::io::prelude::*;
        use tinytemplate::TinyTemplate;
        use itertools::intersperse;
        use crate::Arguments;
        use crate::target::Target;

        pub struct TargetSql {
            pub table            : String,
            pub columns          : Vec<String>,
            pub chunk            : usize,
            pub chunk_insert     : usize,
            pub prefix           : String,
            pub suffix           : String,
            pub with_transaction : bool,
            pub typed            : bool,
        }

        #[derive(Serialize)]
        struct TemplateContext {
            table: String,
        }

        impl Target for TargetSql {
            fn convert(&self, args: Arguments) -> Result<(), io::Error> {
                if !Path::new(args.source.as_str()).exists() {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "CSV file not found"));
                }
        
                let csv_file = File::open(args.source.clone())?;
                let reader = io::BufReader::new(csv_file);
                let csv_reader = csv::ReaderBuilder::new()
                            .has_headers(args.has_headers)
                            .from_reader(reader);
        
                self.generate_sql_file(args, csv_reader)
            }
        }

        impl TargetSql {
            pub fn generate_sql_file(&self, args: Arguments, csv_reader: csv::Reader<io::BufReader<File>>) -> Result<(), io::Error> {
                let sql_file = File::create(&args.target).expect("Unable to create sql file");
                let mut writer = BufWriter::new(sql_file);
        
                let context = &TemplateContext {
                    table: self.table.to_string()
                };
                self.append_file_content(self.prefix.clone(), context, &mut writer)?;
                self.generate_sql(&args, csv_reader, &mut writer)?;
                self.append_file_content(self.suffix.clone(), context, &mut writer)?;
        
                Ok(())
            }
        
            fn generate_sql(&self, args: &Arguments, mut csv_reader: csv::Reader<io::BufReader<File>>, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
                let insert_fields = self.format_fields(self.get_fields(args, csv_reader.headers()?));
        
                let mut chunk_count = 0;
                let mut chunk_insert_count = 0;
                let mut insert_separator = ";\n\n";
        
                if self.with_transaction {
                    write!(writer, "begin transaction")?;
                } else {
                    insert_separator = "";
                }
        
                for record in csv_reader.records() {
                    if chunk_insert_count == 0 {
                        if self.chunk > 0 && chunk_count == self.chunk {
                            write!(writer, ";\n\ncommit;\n\nbegin transaction")?;
                            chunk_count = 0;
                        }
        
                        write!(writer, "{}insert into {} {} values", insert_separator, self.table.as_str(), insert_fields)?;
                        insert_separator = "";
                        chunk_count += 1;
                    }
        
                    match record {
                        Ok(row) => write!(writer, "{}\n{}", insert_separator, self.get_values(&row))?,
                        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e))
                    }
        
                    if self.chunk_insert > 0 {
                        chunk_insert_count += 1;
                        insert_separator = ",";
                        if self.chunk_insert == chunk_insert_count {
                            chunk_insert_count = 0;
                            insert_separator = ";\n\n";
                        }
                    } else {
                        insert_separator = ";\n\n";
                    }
                }
        
                if self.with_transaction {
                    write!(writer, ";\n\ncommit;")?
                } else {
                    write!(writer, ";")?
                }
        
                Ok(())
            }
                
            fn append_file_content(&self, path: String, context: &TemplateContext, writer: &mut BufWriter<File>) -> Result<(), io::Error> {
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
        
            fn get_fields(&self, args: &Arguments, headers: &csv::StringRecord) -> Vec<String> {
                let mut fields: Vec<String> = Vec::new();
                if self.columns.is_empty() && args.has_headers {
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
    
            fn get_values(&self, record: &csv::StringRecord) -> String {
                let mut values = String::new();
                let mut separator = "";
        
                for result in record {
                    values.push_str(separator);
                    if self.typed {
                        values.push_str(&self.get_value(result));
                    } else {
                        values.push_str("'");
                        values.push_str(&result.replace("'", "''"));
                        values.push_str("'");
                    }
                    separator = ", "
                }
        
                format!("({})", values)
            }
        
            fn get_value(&self, result: &str) -> String {
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
    }

    pub mod csv {
        use std::io;
        use crate::target::Target;
        use crate::Arguments;

        pub struct TargetCsv {}

        impl Target for TargetCsv {
            fn convert(&self, _args: Arguments) -> Result<(), io::Error> {
                Ok(())
            }
        }
    }
}