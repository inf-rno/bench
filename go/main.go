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
	res := []map[string]*result{}
	for i := 0; i < c.runs; i++ {
		fmt.Println("RUNNING: ", i+1)
		bench := newBench(c)
		bench.run()
		res = append(res, bench.result())
	}

	fmt.Println("~~~~~~~~~~~~~~~~~~~RESULTS~~~~~~~~~~~~~~~~")

opLoop:
	for _, op := range []string{"SET", "GET"} {
		fmt.Println("OP: ", op)
		min, max := 0, 0
		for i, r := range res {
			r := r[op]
			rmin := res[min][op]
			rmax := res[max][op]
			if r == nil || rmin == nil || rmax == nil {
				continue opLoop
			}
			if r.p99 < rmin.p99 {
				min = i
			}
			if r.p99 > rmax.p99 {
				max = i
			}
		}
		fmt.Println("BEST RUN: ", min+1, "\n", res[min][op].String())
		fmt.Println("WORST RUN: ", max+1, "\n", res[max][op].String())
		if c.outDir != "" {
			f, err := os.Create(c.outDir + "/go_" + op)
			if err != nil {
				panic(err)
			}
			defer f.Close()
			res[max][op].histogram.PercentilesPrint(f, 10, 1000)
		}
	}
}
