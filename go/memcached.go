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
	addr := fmt.Sprintf("%s:%d", c.server, c.port)
	if c.socket != "" {
		addr = c.socket
	}
	mc := gomemcache.New(addr)
	return &memcache{
		client: mc,
		config: c,
	}
}

func (m *memcache) init() {
	if m.config.dataRange != nil {
		if m.config.keyRange != 0 {
			for i := 0; i < m.config.keyRange; i++ {
				u := rand.Intn(m.config.dataRange[1]-m.config.dataRange[0]+1) + m.config.dataRange[0]
				err := m.client.Set(&gomemcache.Item{Key: fmt.Sprintf("%s-%d", m.config.key, i), Value: m.config.dataBytes[:u]})
				if err != nil {
					panic(fmt.Errorf("failed to init memcache: %w", err))
				}
			}
		} else {
			err := m.client.Set(&gomemcache.Item{Key: m.config.key, Value: m.config.dataBytes})
			if err != nil {
				panic(fmt.Errorf("failed to init memcache: %w", err))
			}
		}
	}
}

func (m *memcache) do() (op string, d time.Duration, err error) {
	random := rand.Float64()
	var k string
	if m.config.keyRange != 0 {
		k = fmt.Sprintf("%s-%d", m.config.key, rand.Intn(m.config.keyRange))
	} else {
		k = m.config.key
	}

	if random <= m.config.ratio && m.config.dataRange[0] != 0 {
		op = "SET"
		u := rand.Intn(m.config.dataRange[1]-m.config.dataRange[0]+1) + m.config.dataRange[0]
		start := hrtime.Now()
		err = m.client.Set(&gomemcache.Item{Key: k, Value: m.config.dataBytes[:u]})
		d = hrtime.Now() - start
	} else {
		op = "GET"
		start := hrtime.Now()
		_, err = m.client.Get(k)
		d = hrtime.Now() - start
	}
	return
}
