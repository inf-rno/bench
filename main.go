package main

import (
	"flag"
	"fmt"
	"math/rand"
	"strings"
	"time"

	gomemcache "github.com/bradfitz/gomemcache/memcache"
	redigo "github.com/gomodule/redigo/redis"
	"github.com/loov/hrtime"
	_ "go.uber.org/automaxprocs"
)

type config struct {
	runs        int
	iters       int
	concurrency int
	ratio       float64
	data        int
	server      string
	port        int
	protocol    string
}

func main() {
	runs := flag.Int("x", 3, "number of full test iterations")
	iters := flag.Int("n", 1000, "number of task iterations per goroutine")
	concurrency := flag.Int("c", 1, "number of concurrent goroutines")
	ratio := flag.Float64("r", 0.1, "ratio of ops (eg. sets vs gets)")
	data := flag.Int("d", 32, "size of the data payload in bytes")
	server := flag.String("s", "127.0.0.1", "server address")
	port := flag.Int("p", 6379, "server port")
	protocol := flag.String("P", "redis", "protocol (redis/memcache)")

	flag.Parse()

	run(config{
		runs:        *runs,
		iters:       *iters,
		concurrency: *concurrency,
		ratio:       *ratio,
		data:        *data,
		server:      *server,
		port:        *port,
		protocol:    *protocol,
	})
}

func run(c config) {
	rand.Seed(time.Now().UnixNano())

	res := []map[string]*result{}
	for i := 0; i < c.runs; i++ {
		fmt.Println("RUNNING: ", i+1)
		var t task
		if c.protocol == "redis" {
			t = newRedis(c.server, c.port, c.data, c.ratio)
		} else if c.protocol == "memcache" {
			t = newMemcache(c.server, c.port, c.data, c.ratio)
		} else {
			panic(fmt.Errorf("unknown protocol: %s", c.protocol))
		}
		t.init()

		bench := newBench(c.iters, c.data)
		for j := 0; j < c.iters; j++ {
			op, d, err := t.do()
			if err != nil {
				panic(fmt.Errorf("failed to do task: %w", err))
			}
			bench.appendTime(op, d)
		}
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

type bench struct {
	iters int
	data  int
	times map[string][]time.Duration
}

func newBench(c int, d int) *bench {
	return &bench{
		iters: c,
		data:  d,
		times: map[string][]time.Duration{},
	}
}

func (b *bench) appendTime(op string, d time.Duration) {
	b.times[op] = append(b.times[op], d)
}

func (b *bench) result() map[string]*result {
	resMap := map[string]*result{}
	for op, times := range b.times {
		resMap[op] = newResult(times, b.data)
	}
	return resMap
}

type result struct {
	ops       int
	total     time.Duration
	opsps     float64
	kbps      float64
	gbps      float64
	histogram *hrtime.Histogram
}

func newResult(times []time.Duration, d int) *result {
	opts := hrtime.HistogramOptions{
		BinCount:        10,
		NiceRange:       true,
		ClampMaximum:    0,
		ClampPercentile: 0.999,
	}
	r := &result{
		ops:       len(times),
		histogram: hrtime.NewDurationHistogram(times, &opts),
	}
	for _, t := range times {
		r.total += t
	}
	r.opsps = float64(r.ops) / (float64(r.total) / float64(time.Second))
	r.kbps = float64(d) * r.opsps / 1000
	r.gbps = float64(d) * 8 * r.opsps / 1000000000
	return r
}

func (r *result) StringStats() string {
	return fmt.Sprintf(" ops %d; total %s; ops/sec %.0f; KBps %.2f; Gbps %.2f\n%s", r.ops, r.total, r.opsps, r.kbps, r.gbps, r.histogram.StringStats())
}

func (r *result) String() string {
	return fmt.Sprintf(" ops %d; total %s; ops/sec %.0f; KBps %.2f; Gbps %.2f\n%s", r.ops, r.total, r.opsps, r.kbps, r.gbps, r.histogram.String())
}

type task interface {
	init()
	do() (string, time.Duration, error)
}

type redis struct {
	conn  redigo.Conn
	key   string
	data  string
	ratio float64
}

func newRedis(addr string, port int, data int, ratio float64) task {
	c, err := redigo.DialURL(fmt.Sprintf("redis://%s:%d", addr, port))
	if err != nil {
		panic(fmt.Errorf("failed to connect to redis: %w", err))
	}
	return &redis{
		conn:  c,
		key:   "lol",
		data:  strings.Repeat("x", data),
		ratio: ratio,
	}
}

func (r *redis) init() {
	_, err := r.conn.Do("SET", r.key, r.data)
	if err != nil {
		panic(fmt.Errorf("failed to init set: %w", err))
	}
}

func (r *redis) do() (op string, d time.Duration, err error) {
	rand := rand.Float64()
	args := []any{}
	if rand <= r.ratio {
		op = "SET"
		args = append(args, r.key, r.data)

	} else {
		op = "GET"
		args = append(args, r.key)
	}
	start := hrtime.Now()
	_, err = r.conn.Do(op, args...)
	d = hrtime.Now() - start
	return
}

type memcache struct {
	client *gomemcache.Client
	key    string
	data   []byte
	ratio  float64
}

func newMemcache(addr string, port int, data int, ratio float64) task {
	mc := gomemcache.New(fmt.Sprintf("%s:%d", addr, port))
	return &memcache{
		client: mc,
		key:    "lol",
		data:   []byte(strings.Repeat("x", data)),
		ratio:  ratio,
	}
}

func (m *memcache) init() {
	m.client.Set(&gomemcache.Item{Key: m.key, Value: m.data})
}

func (m *memcache) do() (op string, d time.Duration, err error) {
	rand := rand.Float64()
	if rand <= m.ratio {
		op = "SET"
		start := hrtime.Now()
		m.client.Set(&gomemcache.Item{Key: m.key, Value: m.data})
		d = hrtime.Now() - start
	} else {
		op = "GET"
		start := hrtime.Now()
		_, err = m.client.Get(m.key)
		d = hrtime.Now() - start
	}
	return
}
