//! Internal metrics collection for roea-ai
//!
//! This module provides a simple metrics collection system for
//! monitoring the agent's internal performance and health.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::Serialize;

/// Counter metric - monotonically increasing value
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
    name: String,
    description: String,
}

impl Counter {
    /// Create a new counter
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            name: name.into(),
            description: description.into(),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific amount
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the counter description
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Gauge metric - value that can go up and down
#[derive(Debug)]
pub struct Gauge {
    value: AtomicU64,
    name: String,
    description: String,
}

impl Gauge {
    /// Create a new gauge
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            name: name.into(),
            description: description.into(),
        }
    }

    /// Set the gauge value
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge by 1
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the gauge name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Histogram for tracking distributions of values
#[derive(Debug)]
pub struct Histogram {
    name: String,
    description: String,
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl Histogram {
    /// Create a new histogram with the given buckets
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        buckets: Vec<f64>,
    ) -> Self {
        let counts = (0..=buckets.len())
            .map(|_| AtomicU64::new(0))
            .collect();

        Self {
            name: name.into(),
            description: description.into(),
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// Create a histogram with default latency buckets (in milliseconds)
    pub fn with_latency_buckets(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(
            name,
            description,
            vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0],
        )
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        // Increment total count and sum
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add((value * 1000.0) as u64, Ordering::Relaxed); // Store as micros for precision

        // Find the bucket and increment
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        // Value exceeded all buckets
        self.counts[self.buckets.len()].fetch_add(1, Ordering::Relaxed);
    }

    /// Observe a duration
    pub fn observe_duration(&self, duration: Duration) {
        self.observe(duration.as_secs_f64() * 1000.0); // Convert to ms
    }

    /// Get the total count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum (in the original unit, not micros)
    pub fn sum(&self) -> f64 {
        self.sum.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Get the average
    pub fn average(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            0.0
        } else {
            self.sum() / count as f64
        }
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    histogram: Arc<Histogram>,
}

impl Timer {
    /// Create a new timer
    pub fn new(histogram: Arc<Histogram>) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        self.histogram.observe_duration(self.start.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Timer will record duration when dropped if not explicitly stopped
    }
}

/// Metrics registry for managing all metrics
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or get a counter
    pub fn counter(&self, name: &str, description: &str) -> Arc<Counter> {
        let counters = self.counters.read();
        if let Some(counter) = counters.get(name) {
            return counter.clone();
        }
        drop(counters);

        let mut counters = self.counters.write();
        counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Counter::new(name, description)))
            .clone()
    }

    /// Register or get a gauge
    pub fn gauge(&self, name: &str, description: &str) -> Arc<Gauge> {
        let gauges = self.gauges.read();
        if let Some(gauge) = gauges.get(name) {
            return gauge.clone();
        }
        drop(gauges);

        let mut gauges = self.gauges.write();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Gauge::new(name, description)))
            .clone()
    }

    /// Register or get a histogram
    pub fn histogram(&self, name: &str, description: &str) -> Arc<Histogram> {
        let histograms = self.histograms.read();
        if let Some(histogram) = histograms.get(name) {
            return histogram.clone();
        }
        drop(histograms);

        let mut histograms = self.histograms.write();
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Histogram::with_latency_buckets(name, description)))
            .clone()
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read();
        let gauges = self.gauges.read();
        let histograms = self.histograms.read();

        MetricsSnapshot {
            counters: counters
                .iter()
                .map(|(k, v)| (k.clone(), v.get()))
                .collect(),
            gauges: gauges
                .iter()
                .map(|(k, v)| (k.clone(), v.get()))
                .collect(),
            histograms: histograms
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        HistogramSnapshot {
                            count: v.count(),
                            sum: v.sum(),
                            average: v.average(),
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Snapshot of histogram data
#[derive(Debug, Clone, Serialize)]
pub struct HistogramSnapshot {
    pub count: u64,
    pub sum: f64,
    pub average: f64,
}

/// Snapshot of all metrics
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, u64>,
    pub histograms: HashMap<String, HistogramSnapshot>,
}

/// Global metrics registry
static METRICS: std::sync::OnceLock<MetricsRegistry> = std::sync::OnceLock::new();

/// Get the global metrics registry
pub fn metrics() -> &'static MetricsRegistry {
    METRICS.get_or_init(MetricsRegistry::new)
}

// Pre-defined metrics for roea-agent

/// Counter for total processes tracked
pub fn processes_tracked() -> Arc<Counter> {
    metrics().counter("roea_processes_tracked_total", "Total number of processes tracked")
}

/// Counter for total connections tracked
pub fn connections_tracked() -> Arc<Counter> {
    metrics().counter("roea_connections_tracked_total", "Total number of network connections tracked")
}

/// Counter for total file operations tracked
pub fn file_ops_tracked() -> Arc<Counter> {
    metrics().counter("roea_file_ops_tracked_total", "Total number of file operations tracked")
}

/// Counter for agent detections
pub fn agents_detected() -> Arc<Counter> {
    metrics().counter("roea_agents_detected_total", "Total number of AI agents detected")
}

/// Gauge for currently active processes
pub fn active_processes() -> Arc<Gauge> {
    metrics().gauge("roea_active_processes", "Number of currently active tracked processes")
}

/// Gauge for currently active connections
pub fn active_connections() -> Arc<Gauge> {
    metrics().gauge("roea_active_connections", "Number of currently active network connections")
}

/// Histogram for process monitoring latency
pub fn process_monitor_latency() -> Arc<Histogram> {
    metrics().histogram("roea_process_monitor_latency_ms", "Process monitoring cycle latency in milliseconds")
}

/// Histogram for network monitoring latency
pub fn network_monitor_latency() -> Arc<Histogram> {
    metrics().histogram("roea_network_monitor_latency_ms", "Network monitoring cycle latency in milliseconds")
}

/// Histogram for gRPC request latency
pub fn grpc_request_latency() -> Arc<Histogram> {
    metrics().histogram("roea_grpc_request_latency_ms", "gRPC request latency in milliseconds")
}

/// Counter for errors
pub fn errors_total() -> Arc<Counter> {
    metrics().counter("roea_errors_total", "Total number of errors encountered")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter", "A test counter");
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.inc_by(10);
        assert_eq!(counter.get(), 11);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        assert_eq!(gauge.get(), 0);

        gauge.set(100);
        assert_eq!(gauge.get(), 100);

        gauge.inc();
        assert_eq!(gauge.get(), 101);

        gauge.dec();
        assert_eq!(gauge.get(), 100);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::with_latency_buckets("test_histogram", "A test histogram");

        histogram.observe(5.0);
        histogram.observe(10.0);
        histogram.observe(50.0);

        assert_eq!(histogram.count(), 3);
        assert!((histogram.average() - 21.666).abs() < 0.01);
    }

    #[test]
    fn test_registry() {
        let registry = MetricsRegistry::new();

        let counter1 = registry.counter("test", "test");
        let counter2 = registry.counter("test", "test");

        // Should return the same counter
        counter1.inc();
        assert_eq!(counter2.get(), 1);
    }

    #[test]
    fn test_snapshot() {
        let registry = MetricsRegistry::new();

        registry.counter("counter1", "desc").inc_by(10);
        registry.gauge("gauge1", "desc").set(50);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.counters.get("counter1"), Some(&10));
        assert_eq!(snapshot.gauges.get("gauge1"), Some(&50));
    }

    #[test]
    fn test_predefined_metrics() {
        let p = processes_tracked();
        p.inc();
        assert_eq!(p.get(), 1);

        let c = active_connections();
        c.set(10);
        assert_eq!(c.get(), 10);
    }
}
