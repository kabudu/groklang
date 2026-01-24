package main

import (
	"fmt"
	"time"
)

func fib(n int) int {
	if n < 2 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func main() {
	start := time.Now()
	result := fib(30)
	duration := time.Since(start)
	fmt.Printf("Result: %d\n", result)
	fmt.Printf("Time: %.4fs\n", duration.Seconds())
}
