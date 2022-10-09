# CSVSource

CSVSource is a command line tool written in Rust to convert a CSV file to SQL
statements. It has special features like grouping insert statements in 
transaction chunks and inserting multiple rows with a single insert statement.

## Usage

learn how to use CSVSource with the `--help` argument:

    $ ./csvsource --help

The simplest approach is to pass the argument `--csv` or `-f` followed by a csv file:

    $ ./csvsource --csv data.csv

It generates the `data.sql` file containing the SQL statements. For more 
options, read the [documentation](https://www.hildeberto.com/csvsource/).

## For Developers

  - [Contributing](https://github.com/htmfilho/csvsource/blob/main/CONTRIBUTING.md)
  - [Code of Conduct](https://github.com/htmfilho/csvsource/blob/main/CODE_OF_CONDUCT.md)
  - [License](https://github.com/htmfilho/csvsource/blob/main/LICENSE)