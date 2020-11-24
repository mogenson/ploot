// +build ignore

package main

import (
	"fmt"
	"math"
	"time"
)

func main() {
	start := time.Now()
	for {
		t := time.Since(start).Seconds()
		s := math.Sin(t)
		fmt.Printf("%.3f\n", s+1)
		time.Sleep(100 * time.Millisecond)
	}
}
