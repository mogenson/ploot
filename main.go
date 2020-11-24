package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"strconv"

	ui "github.com/gizak/termui/v3"
	"github.com/gizak/termui/v3/widgets"
)

func main() {
	info, err := os.Stdin.Stat()
	if err != nil {
		log.Fatal(err)
	}

	if info.Mode()&os.ModeNamedPipe == 0 {
		fmt.Println("The command is intended to work with pipes.")
		fmt.Println("Usage: seq 10 | ploot")
		return
	}

	if err := ui.Init(); err != nil {
		log.Fatal(err)
	}
	defer ui.Close()

	width, height := ui.TerminalDimensions()
	plot := widgets.NewPlot()
	plot.Title = "Ploot"
	plot.SetRect(0, 0, width, height)
	plot.Data = make([][]float64, 1)
	plot.Data[0] = make([]float64, 1)

	data := make(chan float64)
	go readStdin(data)

	kbd := ui.PollEvents()
	for {
		select {
		case event := <-kbd:
			switch event.ID {
			case "q", "<C-c>":
				return
			case "<Resize>":
				width, height := ui.TerminalDimensions()
				plot.SetRect(0, 0, width, height)
				ui.Render(plot)
			}
		case num := <-data:
			plot.Data[0] = append(plot.Data[0], num)
			if len(plot.Data[0]) > width {
				plot.Data[0] = plot.Data[0][len(plot.Data[0])-width:]
			}
			ui.Render(plot)
		}
	}
}

func readStdin(data chan float64) {
	scanner := bufio.NewScanner(os.Stdin)
	scanner.Split(bufio.ScanWords)

	for scanner.Scan() {
		word := scanner.Text()
		num, err := strconv.ParseFloat(word, 64)
		if err == nil {
			data <- num
		}
	}
}
