package main

import (
	"C"

	"fmt"
	"main/database"
)
import (
	"context"
	"encoding/json"

	googlesearch "github.com/rocketlaunchr/google-search"
)

var ctx context.Context

func init() {
	ctx = context.Background()
}

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

type SearchResult struct {
	Description string `json:"description"`
	Link        string `json:"link"`
}

//export GoogleSearch
func GoogleSearch(search string) *C.char {
	fmt.Println("GO: fetching results for " + search)
	results, err := googlesearch.Search(ctx, search, googlesearch.SearchOptions{Limit: 5})
	if err == nil {
		fmt.Printf("fetched %d results\n", len(results))
		var resultsList []SearchResult
		for _, result := range results {
			resultsList = append(resultsList, SearchResult{Description: result.Description, Link: result.URL})
		}
		jsonData, _ := json.Marshal(resultsList)
		resultString := string(jsonData)
		fmt.Println(resultString)
		return C.CString("{\"results\":" + resultString + "}")
	}
	return C.CString("{}")
}

func main() {

}
