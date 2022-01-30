# Roma

Roma is a command line tool writen in Rust to convert a CSV file to SQL insert statements. It has special features like grouping insert statements in transaction chunks and inserting multiple rows with a single insert statement.

## Usage

learn how to use Roma with the `--help` argument:

    $ ./roma --help

The simplest approach is to pass the argument `--csv` followed by a csv file:

    $ ./roma --csv data.csv

It generates the `data.sql` file containing the insert statements. For more information, read the [documentation](https://www.hildeberto.com/roma/).