package main

import (
	"log"
	"net/http"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var (
	requestsTotal = prometheus.NewCounter(prometheus.CounterOpts{
		Name: "requests_total",
		Help: "Total number of requests",
	})
	requestDuration = prometheus.NewHistogram(prometheus.HistogramOpts{
		Name:    "request_duration_seconds",
		Help:    "Duration of requests in seconds",
		Buckets: prometheus.DefBuckets,
	})
)

func init() {
	prometheus.MustRegister(requestsTotal)
	prometheus.MustRegister(requestDuration)
}

func updateMetrics() {
	for {
		requestsTotal.Inc()
		requestDuration.Observe(float64(time.Now().UnixNano()) / 1e9)
		time.Sleep(5 * time.Second)
	}
}

func main() {
	go updateMetrics()

	http.Handle("/metrics", promhttp.Handler())
	log.Fatal(http.ListenAndServe(":8080", nil))
}