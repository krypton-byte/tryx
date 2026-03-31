# Performance Guide

## Handler Design

- keep handlers short
- offload heavy CPU work
- avoid blocking I/O in async handlers

## Throughput Tactics

- batch external API calls when possible
- use bounded queue workers
- reduce repeated protobuf parsing

## Memory Considerations

- avoid storing full event objects for long periods
- persist only required identifiers and metadata
- stream large media payloads instead of keeping many in memory

## Latency Debugging

Track:

- event receive timestamp
- handler start/end time
- outbound API call duration

## Capacity Planning

Measure on realistic workloads:

- message rate per minute
- media transfer volume
- callback concurrency
