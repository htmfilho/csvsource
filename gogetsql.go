package main

import (
	"bufio"
	"encoding/csv"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"regexp"
	"strings"
	"text/template"

	"gogetsql/utils"
)

// Command line flags
var flagCSVFile = flag.String("csv", "", "Absolute or relative path to the CSV file. It must include the file name and extension.")
var flagCSVSeparator = flag.String("separator", "comma", "Supported CSV separators: comma, tab.")
var flagSQLFile = flag.String("sql", "", "Absolute or relative path to the location where the SQL file should be saved. It must include the file name and extension.")
var flagIncludeFirstLine = flag.Bool("includefirstline", false, "If present, it considers the first line.")
var flagTable = flag.String("table", "", "Table name. Required.")
var flagColumns = flag.String("columns", "", "Columns of the table. Example: 'id(number),name(text),description(text)'. Required.")
var flagChunkSize = flag.Int("chunk", 0, "The number of sql statements in a transaction scope. 0 is all statements in the same transaction. It must be greater than --chunkinsert.")
var flagInsertChunkSize = flag.Int("chunkinsert", 1, "The number of rows each insert statement covers. It must be equal or smaller than --chunk.")
var flagPrefixFile = flag.String("prefix", "", "Absolute or relative path to a file whose content is put at the beginning of the sql file.")
var flagSuffixFile = flag.String("suffix", "", "Absolute or relative path to a file whose content is put at the end of the sql file.")

var separators = map[string]rune{
	"comma": ',',
	"tab":   '\t',
}

func main() {
	flag.Parse()

	table := figureOutTableName(*flagTable, *flagCSVFile)
	sqlFilePath := figureOutSQLFilePath(table, *flagSQLFile)

	readCSVFile(*flagCSVFile, sqlFilePath, *flagCSVSeparator, table, *flagColumns, *flagIncludeFirstLine)
}

type record struct {
	values []string
}

func readCSVFile(csvFilePath, sqlFilePath, separator, table, flagColumns string, includeFirstLine bool) {
	csvfile, err := os.Open(csvFilePath)
	if err != nil {
		log.Fatalln("Couldn't open the csv file", err)
	}

	reader := csv.NewReader(csvfile)
	reader.Comma = separators[separator]

	var cols []string
	if !includeFirstLine {
		cols, err = reader.Read()
		if err != nil {
			log.Fatalf("Error reading the csv file: %v", err)
		}
	}

	if len(flagColumns) > 0 {
		cols = strings.Split(flagColumns, string(separators[separator]))
	}

	chunkCount := 0
	insertChunkCount := 0

	processedColumns := processColumns(cols)
	var insertChunk []record

	sqlFile, err := os.Create(sqlFilePath)
	if err != nil {
		panic(err)
	}
	defer sqlFile.Close()

	wSQLFile := bufio.NewWriter(sqlFile)

	err = attachFileContent(wSQLFile, *flagPrefixFile, table)
	if err != nil {
		log.Printf("error attaching the prefix file: %v", err)
	}

	for {
		entry, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatal(err)
		}

		if insertChunkCount < *flagInsertChunkSize {
			insertChunk = append(insertChunk, record{
				values: entry,
			})

			insertChunkCount++
			if insertChunkCount == *flagInsertChunkSize {
				if *flagChunkSize > 0 && chunkCount == 0 {
					wSQLFile.WriteString("begin transaction;\n")
				}

				wSQLFile.WriteString(createInsertStatement(insertChunk, table, processedColumns))
				insertChunkCount = 0
				insertChunk = nil
				chunkCount++
			}
		}

		if *flagChunkSize > 0 && chunkCount == *flagChunkSize {
			wSQLFile.WriteString("commit transaction;\n\n")
			chunkCount = 0
		}

		wSQLFile.Flush()
	}

	if *flagChunkSize > 0 && chunkCount > 0 {
		wSQLFile.WriteString("commit transaction;\n")
	}

	if len(insertChunk) > 0 {
		if *flagChunkSize > 0 {
			wSQLFile.WriteString("\nbegin transaction;")
		}

		wSQLFile.WriteString(createInsertStatement(insertChunk, table, processedColumns))

		if *flagChunkSize > 0 {
			wSQLFile.WriteString("\ncommit transaction;\n")
		}
	}

	err = attachFileContent(wSQLFile, *flagSuffixFile, table)
	if err != nil {
		log.Printf("error attaching the suffix file: %v", err)
	}
	wSQLFile.Flush()
}

func figureOutTableName(flagTable, flagCSVFile string) string {
	if len(flagTable) == 0 {
		// If the flag --table is not informed then take only the file name without extension and ignore the rest.
		re := regexp.MustCompile("[^0-9A-Za-z_]*([a-zA-Z_][a-zA-Z0-9_]*).[csvCSV]{3}$")
		match := re.FindStringSubmatch(flagCSVFile)
		if len(match) > 1 {
			return match[1]
		}

		// If there is an error matching the regex then return unnamed.
		return "unnamed"
	}

	// If the flag --table is informed then use it.
	return flagTable
}

func figureOutSQLFilePath(tableName, flagSQLFile string) string {
	if len(flagSQLFile) == 0 {
		return tableName + ".sql"
	}
	return flagSQLFile
}

func attachFileContent(sqlFile *bufio.Writer, pathFile, table string) error {
	type args struct {
		Table string
	}

	if len(pathFile) == 0 {
		return nil
	}

	prefixTemplate, err := template.ParseFiles(pathFile)
	if err != nil {
		return fmt.Errorf("could not open the file: %w", err)
	}

	err = prefixTemplate.Execute(sqlFile, args{Table: table})
	if err != nil {
		return fmt.Errorf("could not apply the %w", err)
	}

	return nil
}

func createInsertStatement(records []record, table string, columns [][]string) string {
	if len(records) == 0 {
		return ""
	}

	insert := ""
	comma := ""
	insert = "insert into"
	insert = fmt.Sprintf("%s %s (", insert, table)

	for _, col := range columns {
		insert = fmt.Sprintf("%s%s %s", insert, comma, col[0])
		comma = ","
	}
	insert = fmt.Sprintf("%s) values \n", insert)

	for i, record := range records {
		insert = fmt.Sprintf("%s  (", insert)
		comma = ""
		for j, column := range columns {
			if column[1] == "text" {
				insert = fmt.Sprintf("%s%s '%s'", insert, comma, sanitize(record.values[j]))
			} else {
				insert = fmt.Sprintf("%s%s %s", insert, comma, record.values[j])
			}
			comma = ","
		}

		if i < len(records)-1 {
			insert = fmt.Sprintf("%s),\n", insert)
		}
	}

	return fmt.Sprintf("%s);\n", insert)
}

func processColumns(columns []string) [][]string {
	numCols := len(columns)
	processed := make([][]string, numCols)

	for i, col := range columns {
		if strings.Contains(col, "(") {
			processed[i] = []string{utils.Before(col, "("), utils.Between(col, "(", ")")}
		} else {
			processed[i] = []string{col, "text"}
		}
	}

	return processed
}

// Remove characters from the string that might be invalid in the sql query.
func sanitize(str string) string {
	return strings.ReplaceAll(str, "'", "''")
}
