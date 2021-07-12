package main

import (
	"flag"
)

func main() {
	port  := flag.Int("port", 8080, "")
	dbURL := flag.String("db", "flexhub:flexhub@tcp(35.229.150.2:3306)/flex?parseTime=true", "")
}
