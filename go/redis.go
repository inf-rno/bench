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
	if r.config.dataStr != "" {
		_, err := r.conn.Do("SET", r.config.key, r.config.dataStr)
		if err != nil {
			panic(fmt.Errorf("failed to init set: %w", err))
		}
	}
}

func (r *redis) do() (op string, d time.Duration, err error) {
	rand := rand.Float64()
	args := []any{}
	if rand <= r.config.ratio && r.config.dataStr != "" {
		op = "SET"
		args = append(args, r.config.key, r.config.dataStr)
	} else {
		op = "GET"
		args = append(args, r.config.key)
	}
	start := hrtime.Now()
	_, err = r.conn.Do(op, args...)
	d = hrtime.Now() - start
	return
}
