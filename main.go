package main

import (
	"fmt"
	"io/ioutil"
	"log"

	pb "./user"
	"github.com/golang/protobuf/proto"
)

func main() {
	p := &pb.User{
		Name: "Alice",
		Age:  20,
	}

	out, err := proto.Marshal(p)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(out)
	if err := ioutil.WriteFile("./go_user.bin", out, 0644); err != nil {
		log.Fatalln("Failed to write:", err)
	}
}
