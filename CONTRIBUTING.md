# How to Contribute

## Common Scenarios

### Did you find a bug?

* **Do not open up a GitHub issue if the bug is a security vulnerability**. Instead, send a direct email to me@hildeberto.com.

* **Ensure the bug was not already reported** by searching on GitHub under [Issues](https://github.com/htmfilho/csvsource/issues).

* If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/htmfilho/csvsource/issues/new). Be sure to include a **title and clear description**, as much relevant information as possible, and a **code sample** or an **executable test case** demonstrating the expected behavior that is not occurring.

* If possible, use the relevant bug report templates to create the issue.

### Did you write a patch that fixes a bug?

* Open a new GitHub pull request with the patch.

* Ensure the PR description clearly describes the problem and solution. Include the relevant issue number if applicable.

### Do you intend to add a new feature or change an existing one?

* Suggest your change in an enhancement issue. When you have a green light from the community, go ahead and start writing code.

### Do you have questions about the source code?

* Ask any question about how to use SpitFHIR by creating an issue and tag it with "question".

### Do you want to contribute to the Rails documentation?

If you tried to proceed according to the documentation and something didn't go as expected, please suggest improvements in the documentation to cover your particular use case.

## Commonly Used Cargo Commands

### Documentation

To generate documentation:

    $ cargo doc

To see the documentation:

    $ cargo doc --open

### Run

    $ cargo run -- --source examples/waterloo_tree_inventory.csv

### Build

    $ cargo build

### Release Build

Targeting the current operating system:

    $ cargo build --release

Targeting other operating systems:

    $ cargo build --target=x86_64-pc-windows-gnu

Installing to run locally, by adding the command to the classpath:

    $ cargo install --path .

## Tests

CSVSource was originally developed for learning purpose before being adopted in production. Unfortunately, it lacks automated tests to make sure all its features still work after changes. That would be great if you could contribute with tests, but while we learn how to test Rust code, please run the following test cases to verify whether the resulting target files are generated as expected:

* `$ cargo run -- --source examples/waterloo_tree_inventory.csv`: validates default behaviors:
   1. it generates a sql file by default
   2. there is an sql insert statement for each line of the csv file
   3. the name of the table is equal to the name of the csv file
   4. the first line of the csv is used to define the names of the columns
   5. all values are defined as string

* `$ cargo run -- --source examples/waterloo_tree_inventory.csv --target different_name.sql`:
   1. it generates the target with a diferent file name from the source
   2. the table name is still the name of the source file

* `$ cargo run -- --source examples/waterloo_tree_inventory.csv --table tree_inventory`:
   1. the table name in the insert statements is different from the source file name



* `$ rm *.sql` to clean up all the generated targets.