# Goma

Goma is a library writen in Go to convert tabular data to SQL insert statements. Its first capability is converting a CSV file into a SQL file.

## CSV to SQL

Usage of `goma`:

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

    $ ./goma --help

    $ ./goma \
        --csv=test_comma.csv \
        --separator=comma \
        --skipfirstline \
        --table=mapping \
        --chunk=5 \
        --chunkinsert=5 \
        --columns='source_code(text),source_description(text),source_system(text),source_version(text),target_code(text),target_description(text),target_system(text),target_version(text),valid_start_date(text),valid_end_date(text),invalid_reason(boolean)'
