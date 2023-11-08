package main

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	_ "go.uber.org/automaxprocs"
)

func main() {
	runs := flag.Int("x", 3, "number of full test iterations")
	iters := flag.Int("n", 10000, "number of task iterations per goroutine")
	concurrency := flag.Int("c", 1, "number of concurrent goroutines")
	ratio := flag.Float64("r", 0.1, "ratio of ops (eg. sets vs gets)")
	key := flag.String("k", "lol", "key/prefix to use")
	keyRange := flag.Int("K", 100000, "key range")
	data := flag.String("d", "100000", "size of the data payload in bytes, specify 0 to not perform any writes. optionally a range (e.g. 100-1000)")
	server := flag.String("s", "127.0.0.1", "server address")
	port := flag.Int("p", 6379, "server port")
	socket := flag.String("S", "", "unix domain socket name")
	protocol := flag.String("P", "redis", "protocol (redis/memcache)")
	out := flag.String("o", "", "output prefix for hdrHistogram files)")

	flag.Parse()

	c := &config{
		runs:        *runs,
		iters:       *iters,
		concurrency: *concurrency,
		ratio:       *ratio,
		key:         *key,
		keyRange:    *keyRange,
		server:      *server,
		port:        *port,
		socket:      *socket,
		protocol:    *protocol,
		out:         *out,
	}

	if strings.Contains(*data, "-") {
		split := strings.Split(*data, "-")
		if len(split) != 2 {
			panic(fmt.Errorf("invalid data range: %s", *data))
		}
		lower, err := strconv.Atoi(split[0])
		if err != nil {
			panic(fmt.Errorf("invalid lower data range: %s", *data))
		}
		upper, err := strconv.Atoi(split[1])
		if err != nil {
			panic(fmt.Errorf("invalid upper data range: %s", *data))
		}
		c.dataRange = []int{lower, upper}
	} else {
		d, err := strconv.Atoi(*data)
		if err != nil {
			panic(fmt.Errorf("invalid data size: %s", *data))
		}
		c.dataRange = []int{d}
	}
	run(c)
}

func run(c *config) {
	minMap, maxMap := map[string]*result{}, map[string]*result{}
	for i := 0; i < c.runs; i++ {
		fmt.Println("RUNNING: ", i+1)
		bench := newBench(c)
		bench.run()
		for op, r := range bench.result() {
			if x, ok := minMap[op]; !ok || r.opsps < x.opsps {
				minMap[op] = r
			}
			if x, ok := maxMap[op]; !ok || r.opsps > x.opsps {
				maxMap[op] = r
			}
		}
		time.Sleep(time.Second)
	}

	fmt.Println("~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~")
	fmt.Println("WORST RUN:")
	for op, r := range minMap {
		fmt.Println(op, ":\n", r.String())
	}

	fmt.Println("BEST RUN:")
	for op, r := range maxMap {
		fmt.Println(op, ":\n", r.String())
		if c.out != "" {
			f, err := os.Create(c.out + "_go_" + op)
			if err != nil {
				panic(err)
			}
			defer f.Close()
			r.histogram.PercentilesPrint(f, 10, 1000)
		}
	}
}
