package utils

import "strings"

// Between returns the substring within the value that is between the strings 'a' and 'b'.
func Between(value string, left string, right string) string {
	posLeft := strings.Index(value, left)
	if posLeft == -1 {
		return ""
	}
	posRight := strings.Index(value, right)
	if posRight == -1 {
		return ""
	}
	posAfterLeft := posLeft + len(left)
	if posAfterLeft >= posRight {
		return ""
	}
	return value[posAfterLeft:posRight]
}

// Before returns the substring within the value that is before string 'a'.
func Before(value string, right string) string {
	posRight := strings.Index(value, right)
	if posRight == -1 {
		return ""
	}
	return value[0:posRight]
}
