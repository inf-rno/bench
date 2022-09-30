package main

import (
	"fmt"
	"math/rand"
	"time"

	gomemcache "github.com/bradfitz/gomemcache/memcache"
	"github.com/loov/hrtime"
)

type memcache struct {
	client *gomemcache.Client
	config *config
}

func newMemcache(c *config) task {
	mc := gomemcache.New(fmt.Sprintf("%s:%d", c.server, c.port))
	return &memcache{
		client: mc,
		config: c,
	}
}

func (m *memcache) init() {
	if len(m.config.dataBytes) != 0 {
		m.client.Set(&gomemcache.Item{Key: m.config.key, Value: m.config.dataBytes})
	}
}

func (m *memcache) do() (op string, d time.Duration, err error) {
	rand := rand.Float64()
	if rand <= m.config.ratio && len(m.config.dataBytes) != 0 {
		op = "SET"
		start := hrtime.Now()
		m.client.Set(&gomemcache.Item{Key: m.config.key, Value: m.config.dataBytes})
		d = hrtime.Now() - start
	} else {
		op = "GET"
		start := hrtime.Now()
		_, err = m.client.Get(m.config.key)
		d = hrtime.Now() - start
	}
	return
}
