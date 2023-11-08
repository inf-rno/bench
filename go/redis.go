package main

import (
	"fmt"
	"math/rand"
	"time"

	redigo "github.com/gomodule/redigo/redis"
	"github.com/loov/hrtime"
)

type redis struct {
	conn   redigo.Conn
	config *config
}

func newRedis(c *config) task {
	conn, err := redigo.DialURL(fmt.Sprintf("redis://%s:%d", c.server, c.port))
	if err != nil {
		panic(fmt.Errorf("failed to connect to redis: %w", err))
	}
	return &redis{
		conn:   conn,
		config: c,
	}
}

func (r *redis) init() {
	if r.config.dataRange != nil {
		if r.config.keyRange != 0 {
			for i := 0; i < r.config.keyRange; i++ {
				u := rand.Intn(r.config.dataRange[1]-r.config.dataRange[0]+1) + r.config.dataRange[0]
				_, err := r.conn.Do("SET", fmt.Sprintf("%s-%d", r.config.key, i), string(r.config.dataBytes[:u]))
				if err != nil {
					panic(fmt.Errorf("failed to init set: %w", err))
				}
			}
		} else {
			_, err := r.conn.Do("SET", r.config.key, string(r.config.dataBytes))
			if err != nil {
				panic(fmt.Errorf("failed to init set: %w", err))
			}
		}
	}
}

func (r *redis) do() (op string, d time.Duration, err error) {
	random := rand.Float64()
	args := []any{}
	var k string
	if r.config.keyRange != 0 {
		k = fmt.Sprintf("%s-%d", r.config.key, rand.Intn(r.config.keyRange))
	} else {
		k = r.config.key
	}

	if random <= r.config.ratio && r.config.dataRange[0] != 0 {
		op = "SET"
		u := rand.Intn(r.config.dataRange[1]-r.config.dataRange[0]+1) + r.config.dataRange[0]
		args = append(args, k, string(r.config.dataBytes[:u]))
	} else {
		op = "GET"
		args = append(args, k)
	}
	start := hrtime.Now()
	_, err = r.conn.Do(op, args...)
	d = hrtime.Now() - start
	return
}
