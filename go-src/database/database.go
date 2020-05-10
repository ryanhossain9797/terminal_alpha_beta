package database

import (
	"C"
	"fmt"

	"context"

	"cloud.google.com/go/firestore"
	"google.golang.org/api/iterator"
	"google.golang.org/api/option"
)
import "encoding/json"

func init() {
	fmt.Println("initialized storage")
	ctx = context.Background()
	loadDB()
}

var (
	People   = make(map[string]Person)
	ctx      context.Context
	PeopleDB *firestore.CollectionRef
	InfoDB   *firestore.CollectionRef
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
	InfoDB = client.Collection("info")
}

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

type Info struct {
	Title string `firestore:"title" json:"title,omitempty"`
	Info  string `firestore:"info"  json:"info,omitempty"`
}

func FindInfoFromDB(title string, pass string) string {
	fmt.Println("INFO_FROM_DB: title " + title + " pass " + pass)
	result := InfoDB.Where("title", "==", title).Where("pass", "==", pass).Documents(ctx)

	var info Info
	personSnapshot, err := result.Next()
	if err == iterator.Done {

		return "{}"
	} else if err != nil {
		fmt.Println(err)
		return "{}"
	}
	personSnapshot.DataTo(&info)
	jsonData, err := json.Marshal(info)
	if err != nil {
		fmt.Println(err)
		return "{}"
	}
	return string(jsonData)
}
