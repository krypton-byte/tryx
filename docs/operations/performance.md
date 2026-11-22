# :material-speedometer: Performance Guide

Optimize for predictable latency and stable memory under burst traffic.

## Built-in Low-Overhead Architecture

Tryx minimizes Python/GIL overhead and memory churn in the Rust layer:

1. **Event-Interest Filtering**: `Tryx` registers only the event types that have Python `@app.on(...)` handlers; unsubscribed events are dropped inside Rust without acquiring the GIL.
2. **Shortcut Getters & Lazy Parsing**: Read `event.text`, `event.chat`, `event.sender`, `event.id`, and `event.has_media` on `EvMessage` without allocating `MessageData` or `raw_proto`. `EvHistorySync` keeps payloads compressed until `raw_bytes()` or `proto` is requested, and decompresses off-GIL.
3. **Zero-Roundtrip & Streaming Media**: `client.download_media(event)` and `client.download_media_to_file(event, path)` read media descriptors directly from Rust `Arc<Message>` without serializing protobuf back from Python.
4. **Bounded Signal Caches**: Pass `signal_cache_max_entries` to `Tryx(backend, signal_cache_max_entries=...)` to cap Signal ratchet memory on high-volume bots.
5. **Pooled Storage & Shared Tokio Runtime**: `SqliteStore` caches its `SqliteDatabase` pool in a `OnceCell`, and all async Python futures share `pyo3_async_runtimes::tokio::get_runtime()`.

## Handler Design Rules

1. keep event handlers short and non-blocking
2. offload heavy CPU to workers
3. avoid synchronous I/O in async handler path

!!! tip "Target"
	Keep p95 handler latency low enough that user-facing replies remain responsive under burst load.

## Throughput Tactics

=== "Compute"
	- cache repeated lookup results
	- avoid repeated protobuf parsing for the same payload

=== "I/O"
	- batch outbound calls where safe
	- use bounded worker queues

=== "Media"
	- stream large blobs to disk/object storage
	- avoid retaining many blobs in process memory

## Measurement Baseline

Track at minimum:

- event receive timestamp
- handler start/end timestamp
- outbound API duration
- queue depth (if queue used)
- memory watermark

## Profiling Workflow

1. capture trace under realistic traffic.
2. identify top handler hotspots.
3. optimize one hotspot at a time.
4. rerun same workload and compare metrics.

## Capacity Planning Checklist

- message rate per minute
- media download/upload volume
- active callback concurrency
- reconnect frequency

!!! warning "False optimization"
	Optimize after measuring. Guess-based micro-optimizations often increase complexity without improving bottlenecks.

## Related Docs

- [Reliability](reliability.md)
- [Media Workflows](../tutorials/media-workflows.md)
- [Security Practices](security.md)
