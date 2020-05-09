package main

import (
	"C"

	"context"
	"fmt"

	"cloud.google.com/go/firestore"
	"github.com/schollz/closestmatch"
	"google.golang.org/api/iterator"
	"google.golang.org/api/option"
)

func init() {
	fmt.Println("initialized storage")
	ctx = context.Background()
	loadDB()
}

var (
	People   = make(map[string]Person)
	ctx      context.Context
	PeopleDB *firestore.CollectionRef
)

type Person struct {
	Name        string `firestore:"name"`
	Description string `firestore:"description"`
}

func loadDB() {
	client, err := firestore.NewClient(ctx, "terminal-alpha-and-beta", option.WithCredentialsFile("./creds.json"))
	if err != nil {
		fmt.Println("error occured")
		fmt.Println(err)
		fmt.Println("error end")
	}
	PeopleDB = client.Collection("people")
}

//export GetPerson
func GetPerson(name string) *C.char {
	fmt.Println(name + " HELLO FROM GO")

	fmt.Println("identifying '" + name + "'")
	person, err := FindPersonFromDB(name)
	if err != nil {
		people := GetPeopleFromDB()
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
			person, _ := FindPersonFromDB(assumedName)
			return C.CString("We couldn't find the exact person\nbut we found " + person.Name + "\n" + person.Description)

		} else {
			return C.CString("Human couldn't be identified, tagged as potential threat")

		}
	} else {
		return C.CString(person.Description)
	}
}

func main() {}

func GetPeopleFromDB() []Person {
	peopleDocs := PeopleDB.Documents(ctx)
	var people []Person
	for {
		personSnapshot, err := peopleDocs.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			fmt.Println(err)
			return people
		}
		var person Person
		personSnapshot.DataTo(&person)
		people = append(people, person)
	}
	return people
}

func FindPersonFromDB(name string) (Person, error) {
	result := PeopleDB.Where("name", "==", name).Documents(ctx)

	var person Person
	personSnapshot, err := result.Next()
	if err == iterator.Done {

		return person, iterator.Done
	} else if err != nil {
		fmt.Println(err)
		return person, err
	}
	personSnapshot.DataTo(&person)
	return person, nil
}
