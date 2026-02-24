---
layout: default
title: Vetis - Benchmarks
nav_order: 4
---

# Benchmarks

Go to `examples/simple` and run:

```bash
cargo run --
```

In another terminal tab, run:

```bash
oha -c 128 -z 10s https://localhost:8443/hello --insecure
```

Output might be something like below, please not log level
has huge impact on performance.

```text
Summary:
  Success rate:	100.00%
  Total:	10001.2078 ms
  Slowest:	940.3868 ms
  Fastest:	0.0414 ms
  Average:	0.6897 ms
  Requests/sec:	184706.9910

  Total data:	35.23 MiB
  Size/request:	20 B
  Size/sec:	3.52 MiB

Response time histogram:
    0.041 ms [1]       |
   94.076 ms [1847147] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  188.110 ms [10]      |
  282.145 ms [12]      |
  376.180 ms [13]      |
  470.214 ms [12]      |
  564.249 ms [13]      |
  658.283 ms [14]      |
  752.318 ms [13]      |
  846.352 ms [10]      |
  940.387 ms [23]      |

Response time distribution:
  10.00% in 0.3281 ms
  25.00% in 0.4463 ms
  50.00% in 0.6010 ms
  75.00% in 0.8024 ms
  90.00% in 1.0391 ms
  95.00% in 1.2168 ms
  99.00% in 1.6595 ms
  99.90% in 2.4017 ms
  99.99% in 3.4211 ms


Details (average, fastest, slowest):
  DNS+dialup:	522.2068 ms, 27.9548 ms, 939.1788 ms
  DNS-lookup:	0.0147 ms, 0.0008 ms, 0.1313 ms

Status code distribution:
  [200] 1847268 responses
```
