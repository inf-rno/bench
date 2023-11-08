package main

import "fmt"

type config struct {
	runs        int
	iters       int
	concurrency int
	ratio       float64
	key         string
	keyRange    int
	dataRange   []int
	server      string
	port        int
	socket      string
	protocol    string
	out         string
}

func (c *config) newTask() task {
	if c.protocol == "redis" {
		return newRedis(c)
	} else if c.protocol == "memcache" {
		return newMemcache(c)
	} else {
		panic(fmt.Errorf("unknown protocol: %s", c.protocol))
	}
}
