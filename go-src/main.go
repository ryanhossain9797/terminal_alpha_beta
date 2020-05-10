package main

import (
	"C"

	"fmt"
	"main/database"

	"github.com/schollz/closestmatch"
)

//export GetPerson
func GetPerson(name string) *C.char {
	fmt.Println(name + " HELLO FROM GO")

	fmt.Println("identifying '" + name + "'")
	person, err := database.FindPersonFromDB(name)
	if err != nil {
		people := database.GetPeopleFromDB()
		if people == nil {
			return C.CString("Human couldn't be identified, tagged as potential threat")
		}
		names := []string{}
		for _, person := range people {
			names = append(names, person.Name)
		}
		bagsizes := []int{len(name) / 5 * 3}
		cm := closestmatch.New(names, bagsizes)
		assumedName := cm.Closest(name)
		if len(assumedName) > 0 {
			person, _ := database.FindPersonFromDB(assumedName)
			return C.CString("We couldn't find the exact person\nbut we found " + person.Name + "\n" + person.Description)

		} else {
			return C.CString("Human couldn't be identified, tagged as potential threat")

		}
	} else {
		return C.CString(person.Description)
	}
}

func main() {}
