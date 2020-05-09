package main

import (
	"C"
	"fmt"
)

//export GetPerson
func GetPerson(name string) *C.char {
	fmt.Println(name + " HELLO FROM GO")
	return C.CString(name + " is just some human idiot")
}

func main() {}
