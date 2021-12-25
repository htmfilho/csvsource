# Roma

Roma is a library writen in Rust to convert a CSV file to SQL insert statements. It has special features like grouping insert statements in transaction chunks and inserting multiple rows with a single insert statement.

## CSV to SQL

Usage of `roma`:

  - _--chunk_ (int): 
        The number of sql statements in a transaction scope. 0 is all statements in the same transaction. It must be greater than --chunkinsert.
  - _--chunkinsert_ (int):
        The number of rows each insert statement covers. It must be equal or smaller than --chunk. (default 1)
  - _--columns_ (string):
        Columns of the table. Example: 'id(number),name(text),description(text)'. If not informed, it tries to use the first line of the CSV.
  - _--csv_ (string):
        Path to the CSV file. Required.
  - _--separator_ (string):
        Supported CSV separators: comma, tab. (default "comma")
  - _--includefirstline_:
        If present, it considers that the first line also contains data.
  - _--table_ (string):
        Table name. Required.

Examples:

    $ ./roma --help

    $ ./roma \
        --csv=test_comma.csv \
        --separator=comma \
        --skipfirstline \
        --table=mapping \
        --chunk=5 \
        --chunkinsert=5 \
        --columns='source_code(text),source_description(text),source_system(text),source_version(text),target_code(text),target_description(text),target_system(text),target_version(text),valid_start_date(text),valid_end_date(text),invalid_reason(boolean)'
