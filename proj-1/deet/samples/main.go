package main

import (
	"fmt"
	"time"
)

func main() {
	printTable()
	fmt.Println("Process Exited")
}

func printTable() {
	for i := 0; i < 10; i++ {
		fmt.Println(i)
		time.Sleep(time.Second)
	}
}