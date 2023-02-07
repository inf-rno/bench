package main

import (
	"flag"
	"fmt"
	"math/rand"
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
			h := r.histogram
			if h.P99 < rmin.histogram.P99 {
				min = i
			}
			if h.P99 > rmax.histogram.P99 {
				max = i
			}
		}
		fmt.Println("BEST RUN: ", min+1, "\n", res[min][op].StringStats())
		fmt.Println("WORST RUN: ", max+1, "\n", res[max][op].String())

	}
}
