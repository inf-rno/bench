package main

import (
	"fmt"
	"time"

	"github.com/HdrHistogram/hdrhistogram-go"
	"golang.org/x/sync/errgroup"
)

type bench struct {
	config  *config
	resChan chan taskResult
	times   map[string][]time.Duration
}

func newBench(c *config) *bench {
	return &bench{
		config:  c,
		resChan: make(chan taskResult, c.concurrency*c.iters),
		times:   map[string][]time.Duration{},
	}
}

func (b *bench) run() {
	b.config.newTask().init()
	go b.spawn()
	for r := range b.resChan {
		b.times[r.op] = append(b.times[r.op], r.duration)
	}
}

func (b *bench) spawn() {
	g := new(errgroup.Group)
	g.SetLimit(b.config.concurrency)

	for i := 0; i < b.config.concurrency; i++ {
		g.Go(func() error {
			t := b.config.newTask()
			for i := 0; i < b.config.iters; i++ {
				op, d, err := t.do()
				if err != nil {
					return fmt.Errorf("failed to do task: %w", err)
				}
				b.resChan <- taskResult{op, d}
			}
			return nil
		})
	}

	if err := g.Wait(); err != nil {
		panic(fmt.Errorf("task run failed :%w", err))
	}
	close(b.resChan)
}

func (b *bench) result() map[string]*result {
	resMap := map[string]*result{}
	var d int
	if len(b.config.dataRange) == 1 {
		d = b.config.dataRange[0]
	} else {
		d = 0
	}
	for op, times := range b.times {
		resMap[op] = newResult(times, d)
	}
	return resMap
}

type result struct {
	ops       int
	total     time.Duration
	opsps     float64
	kbps      float64
	gbps      float64
	p99       time.Duration
	histogram *hdrhistogram.Histogram
}

func newResult(times []time.Duration, d int) *result {
	r := &result{
		ops:       len(times),
		histogram: hdrhistogram.New(10, 60000000, 3),
	}
	for _, t := range times {
		r.total += t
		r.histogram.RecordValue(t.Microseconds())
	}
	r.opsps = float64(r.ops) / (float64(r.total) / float64(time.Second))
	r.kbps = float64(d) * r.opsps / 1000
	r.gbps = float64(d) * 8 * r.opsps / 1000000000
	r.p99 = time.Duration(r.histogram.ValueAtPercentile(99)) * time.Microsecond
	return r
}

func (r *result) String() string {
	return fmt.Sprintf(" ops %d; total %s; ops/sec %.0f; p99: %s, KBps %.2f; Gbps %.2f\n", r.ops, r.total, r.opsps, r.p99, r.kbps, r.gbps)
}

type task interface {
	init()
	do() (string, time.Duration, error)
}

type taskResult struct {
	op       string
	duration time.Duration
}
