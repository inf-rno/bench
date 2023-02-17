package main

import (
	"flag"
	"fmt"
	"math/rand"
	"os"
	"strings"
	"time"

	_ "go.uber.org/automaxprocs"
)

func main() {
	runs := flag.Int("x", 1, "number of full test iterations")
	iters := flag.Int("n", 1000, "number of task iterations per goroutine")
	concurrency := flag.Int("c", 1, "number of concurrent goroutines")
	ratio := flag.Float64("r", 0.1, "ratio of ops (eg. sets vs gets)")
	key := flag.String("k", "lol", "key/prefix to use")
	data := flag.Int("d", 32, "size of the data payload in bytes, specify 0 to not perform any writes")
	server := flag.String("s", "127.0.0.1", "server address")
	port := flag.Int("p", 6379, "server port")
	protocol := flag.String("P", "redis", "protocol (redis/memcache)")
	outDir := flag.String("o", "", "directory for hdrHistogram output)")

	flag.Parse()

	rand.Seed(time.Now().UnixNano())

	c := &config{
		runs:        *runs,
		iters:       *iters,
		concurrency: *concurrency,
		ratio:       *ratio,
		key:         *key,
		server:      *server,
		port:        *port,
		protocol:    *protocol,
		outDir:      *outDir,
	}
	c.dataStr = strings.Repeat("x", *data)
	c.dataBytes = []byte(c.dataStr)
	run(c)
}

func run(c *config) {
	minMap, maxMap := map[string]*result{}, map[string]*result{}
	for i := 0; i < c.runs; i++ {
		fmt.Println("RUNNING: ", i+1)
		bench := newBench(c)
		bench.run()
		for op, r := range bench.result() {
			if x, ok := minMap[op]; !ok || r.p99 < x.p99 {
				minMap[op] = r
			}
			if x, ok := maxMap[op]; !ok || r.p99 > x.p99 {
				maxMap[op] = r
			}
		}
		time.Sleep(time.Second)
	}

	fmt.Println("~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~")
	fmt.Println("WORST RUN:")
	for op, r := range maxMap {
		fmt.Println(op, ":\n", r.String())
	}

	fmt.Println("BEST RUN:")
	for op, r := range minMap {
		fmt.Println(op, ":\n", r.String())
		if c.outDir != "" {
			f, err := os.Create(c.outDir + "/go_" + op)
			if err != nil {
				panic(err)
			}
			defer f.Close()
			r.histogram.PercentilesPrint(f, 10, 1000)
		}
	}
}
