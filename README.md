# Roma

Roma is a command line tool written in Rust to convert a CSV file to SQL
statements. It has special features like grouping insert statements in 
transaction chunks and inserting multiple rows with a single insert statement.

## Usage

learn how to use Roma with the `--help` argument:

    $ ./roma --help

The simplest approach is to pass the argument `--csv` or `-f` followed by a csv file:

    $ ./roma --csv data.csv

It generates the `data.sql` file containing the SQL statements. For more 
options, read the [documentation](https://www.hildeberto.com/roma/).