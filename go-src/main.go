package main

import (
	"C"

	"fmt"
	"main/database"
)
import "encoding/json"

//export GetPerson
func GetPerson(name string) *C.char {
	fmt.Println(name + " HELLO FROM GO")

	fmt.Println("identifying '" + name + "'")
	person, err := database.FindPersonFromDB(name)
	if err != nil {
		fmt.Println(err)
		return C.CString("{}")
	}
	personString, err := json.Marshal(person)
	if err != nil {
		fmt.Println(err)
		return C.CString("{}")
	}
	return C.CString("{\"person\":" + string(personString) + "}")
}

//export GetPeople
func GetPeople() *C.char {
	fmt.Println("HELLO FROM GO")

	fmt.Println("fetching all people")
	people := database.GetPeopleFromDB()

	personString, err := json.Marshal(people)
	if err != nil {
		fmt.Println(err)
		return C.CString("{}")
	}
	return C.CString("{\"people\":" + string(personString) + "}")
}

//export GetInfo
func GetInfo(title string, pass string) *C.char {
	fmt.Println("GETINFO: Getting info for " + title)

	data := database.FindInfoFromDB(title, pass)

	fmt.Println("GETINFO: got data " + data)

	return C.CString(data)

}

func main() {

}
