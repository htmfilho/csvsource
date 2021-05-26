package main

import (
	"reflect"
	"testing"
)

func Test_sanitize(t *testing.T) {
	type args struct {
		str string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "Replaced ' by ''",
			args: args{
				str: "wendy's",
			},
			want: "wendy''s",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := sanitize(tt.args.str); got != tt.want {
				t.Errorf("sanitize() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_processColumns(t *testing.T) {
	type args struct {
		columns []string
	}
	tests := []struct {
		name string
		args args
		want [][]string
	}{
		{
			name: "Process columns without types",
			args: args{
				columns: []string{"test1", "test2"},
			},
			want: [][]string{
				{"test1", "text"},
				{"test2", "text"},
			},
		},
		{
			name: "Process columns with types",
			args: args{
				columns: []string{"test1(text)", "test2(number)"},
			},
			want: [][]string{
				{"test1", "text"},
				{"test2", "number"},
			},
		},
		{
			name: "Process columns with types and without types",
			args: args{
				columns: []string{"test1", "test2(number)"},
			},
			want: [][]string{
				{"test1", "text"},
				{"test2", "number"},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := processColumns(tt.args.columns); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("processColumns() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_figureOutTableName(t *testing.T) {
	type args struct {
		flagTable   string
		flagCSVFile string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "The CSV file name as table name",
			args: args{
				flagTable:   "",
				flagCSVFile: "/9_CsvTEST_TABLE/a.csV",
			},
			want: "a",
		},
		{
			name: "The --table flag is defined",
			args: args{
				flagTable:   "mandalorian",
				flagCSVFile: "/9_CsvTEST_TABLE/a.csV",
			},
			want: "mandalorian",
		},
		{
			name: "There CSV file is invalid",
			args: args{
				flagTable:   "",
				flagCSVFile: ".csv",
			},
			want: "unnamed",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := figureOutTableName(tt.args.flagTable, tt.args.flagCSVFile); got != tt.want {
				t.Errorf("figureOutTableName() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_figureOutSQLFilePath(t *testing.T) {
	type args struct {
		tableName   string
		flagSQLFile string
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "SQL filename based on the table name.",
			args: args{
				tableName: "mapping",
			},
			want: "mapping.sql",
		},
		{
			name: "SQL filename based on the flag --sql",
			args: args{
				tableName:   "mapping",
				flagSQLFile: "path/to/sql/file.sql",
			},
			want: "path/to/sql/file.sql",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := figureOutSQLFilePath(tt.args.tableName, tt.args.flagSQLFile); got != tt.want {
				t.Errorf("figureOutSQLFilePath() = %v, want %v", got, tt.want)
			}
		})
	}
}
