package main

import (
	"fmt"
	"math/rand"
	"strings"
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
	if m.config.dataRange[0] != 0 {
		if m.config.keyRange != 0 {
			for i := 0; i < m.config.keyRange; i++ {
				if len(m.config.dataRange) == 1 && m.config.dataRange[0] != 0 {
					v := []byte(strings.Repeat("x", m.config.dataRange[0]))
					m.client.Set(&gomemcache.Item{Key: fmt.Sprintf("%s-%d", m.config.key, i), Value: v})
				} else if len(m.config.dataRange) == 2 {
					m.client.Set(&gomemcache.Item{Key: fmt.Sprintf("%s-%d", m.config.key, i), Value: []byte(strings.Repeat("x", rand.Intn(m.config.dataRange[1]-m.config.dataRange[0]+1)+m.config.dataRange[1]))})
				}
			}
		} else {
			m.client.Set(&gomemcache.Item{Key: m.config.key, Value: []byte(strings.Repeat("x", m.config.dataRange[0]))})
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
		var val []byte
		if len(m.config.dataRange) == 1 {
			val = []byte(strings.Repeat("x", m.config.dataRange[0]))
		} else {
			val = []byte(strings.Repeat("x", rand.Intn(m.config.dataRange[1]-m.config.dataRange[0]+1)+m.config.dataRange[1]))
		}
		start := hrtime.Now()
		m.client.Set(&gomemcache.Item{Key: k, Value: val})
		d = hrtime.Now() - start
	} else {
		op = "GET"
		start := hrtime.Now()
		_, err = m.client.Get(k)
		d = hrtime.Now() - start
	}
	return
}
