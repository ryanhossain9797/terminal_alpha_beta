package main

import (
	"C"

	"fmt"
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

type SearchResult struct {
	Description string `json:"description"`
	Link        string `json:"link"`
}

//GoogleSearch ...
//export GoogleSearch
func GoogleSearch(search string) *C.char {
	fmt.Println("GOOGLE_SEARCH_GO: fetching results for " + search)
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
func main() {}
