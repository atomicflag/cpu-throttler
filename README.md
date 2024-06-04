# CPU Throttler

CPU throttler for Linux intended to keep Intel i9 13900KF (and similar) CPUs cool under heavy load. Should keep the CPU temperature below 80C regardless of workload.

The throttling curve looks something like this:

```
Temperature    <75      75      80      100
-------------------------------------------
Throttle       100      75      66      66
```