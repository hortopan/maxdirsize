# maxdirsize
Periodically watch a directory and keep it at maximum size by removing old files based file modification/creation time.
Can be used as docker/k8s daemonset, etc for cache cleanup, etc.

Written in Rust 🦀.

##  Configure via env variables

```bash
export MAX_SIZE_MB=128
export DIRECTORY=/folder-to-watch
export INTERVAL_SECONDS=60
export RUST_LOG=info/debug/error/warn
```

## Docker image available (arm64, amd64)

```
docker pull hortopan/maxdirsize:0.0.1
```
