# Performance Guide

Optimize for predictable latency and stable memory under burst traffic.

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
