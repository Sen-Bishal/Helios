# LITHOS Docker Deployment Guide

## Quick Start

### Build and Run
```bash
docker build -t lithos:latest .
docker run --rm lithos:latest
```

### Using Docker Compose
```bash
docker-compose up lithos-simulator
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (debug, info, warn, error) |
| `LITHOS_TICK_DURATION_US` | `1` | Simulation tick duration in microseconds |
| `LITHOS_BURST_DROPLETS` | `100` | Number of droplets per simulation burst |
| `LITHOS_SIMULATION_DURATION_MS` | `50` | Total simulation duration in milliseconds |

### Custom Configuration
```bash
docker run --rm \
  -e LITHOS_TICK_DURATION_US=10 \
  -e LITHOS_BURST_DROPLETS=200 \
  -e LITHOS_SIMULATION_DURATION_MS=100 \
  lithos:latest
```

## Volume Mounts

### Export Output Data
```bash
docker run --rm \
  -v $(pwd)/output:/app/output \
  lithos:latest
```

### Export Performance Metrics
```bash
docker run --rm \
  -v $(pwd)/metrics:/app/metrics \
  lithos:latest --export-metrics
```

## Multi-Stage Build Details

### Stage 1: Builder
- Base: `rust:1.75-slim`
- Installs build dependencies (pkg-config, libssl-dev)
- Compiles LITHOS in release mode
- Output: Optimized binary at `/usr/src/lithos/target/release/lithos`

### Stage 2: Runtime
- Base: `debian:bookworm-slim`
- Minimal runtime dependencies (libssl3, ca-certificates)
- Non-root user execution (UID 1000)
- Final image size: ~100MB (vs 2GB+ with full Rust toolchain)

## Resource Limits

Default limits in docker-compose.yml:
- CPU: 2-4 cores
- Memory: 2-4 GB

Adjust based on simulation scale:
```yaml
deploy:
  resources:
    limits:
      cpus: '8'
      memory: 8G
```

## Performance Considerations

### Current Baseline
- Simulation runs at 0.003x realtime
- p95 tick time: ~2022 μs
- Total time: 16.6s for 50ms simulation

### Container Overhead
Docker adds minimal overhead (<5%) for CPU-bound workloads like LITHOS.

### Optimization Recommendations
1. Use `--cpus` flag to allocate multiple cores
2. Mount `/dev/shm` for shared memory if implementing parallel processing
3. Use `--memory-swap=-1` to disable swap for consistent performance

## Production Deployment

### Health Checks
Add to Dockerfile:
```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD pgrep lithos || exit 1
```

### Logging
Redirect logs to volume:
```bash
docker run --rm \
  -v $(pwd)/logs:/app/logs \
  lithos:latest > /app/logs/simulation.log 2>&1
```

### Orchestration with Kubernetes
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: lithos-simulator
spec:
  containers:
  - name: lithos
    image: lithos:latest
    resources:
      requests:
        memory: "2Gi"
        cpu: "2"
      limits:
        memory: "4Gi"
        cpu: "4"
    env:
    - name: LITHOS_SIMULATION_DURATION_MS
      value: "100"
```

## Troubleshooting

### Container Exits Immediately
Check logs:
```bash
docker logs <container_id>
```

### Performance Issues
Monitor resource usage:
```bash
docker stats lithos-sim
```

### Build Failures
Clean build cache:
```bash
docker builder prune
docker build --no-cache -t lithos:latest .
```

## Next Steps

1. Implement BVH spatial acceleration before Phase 3
2. Add multi-threading support (requires `--cap-add=SYS_NICE`)
3. Export metrics in Prometheus format for monitoring
4. Create Grafana dashboard for real-time visualization