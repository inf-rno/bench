package main

import "fmt"

type config struct {
	runs        int
	iters       int
	concurrency int
	ratio       float64
	key         string
	dataStr     string
	dataBytes   []byte
	server      string
	port        int
	protocol    string
	outDir      string
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
