package utils

import "testing"

type args struct {
	value string
	left  string
	right string
}

func TestBetween(t *testing.T) {
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "If the left and right string are found in the value",
			args: args{
				value: "canada",
				left:  "can",
				right: "da",
			},
			want: "a",
		},
		{
			name: "If the left is not found in the value",
			args: args{
				value: "canada",
				left:  "ce",
				right: "da",
			},
			want: "",
		},
		{
			name: "If the right is not found in the value",
			args: args{
				value: "canada",
				left:  "can",
				right: "de",
			},
			want: "",
		},
		{
			name: "If the left ends at the right position in the value",
			args: args{
				value: "canada",
				left:  "can",
				right: "ada",
			},
			want: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := Between(tt.args.value, tt.args.left, tt.args.right); got != tt.want {
				t.Errorf("Between() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestBefore(t *testing.T) {
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "If the right string is found in the value",
			args: args{
				value: "testing",
				right: "ing",
			},
			want: "test",
		},
		{
			name: "If the right string is not found in the value",
			args: args{
				value: "testinn",
				right: "ing",
			},
			want: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := Before(tt.args.value, tt.args.right); got != tt.want {
				t.Errorf("Before() = %v, want %v", got, tt.want)
			}
		})
	}
}
