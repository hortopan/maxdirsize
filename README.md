# maxdirsize
Service that watches a directory and keeps it at maximum size while removing files based file modification/creation time.
Can be used as docker/k8s daemonset, etc for cache cleanup, etc.

Written in Rust 🦀.

##  Configure via env variables

```bash
export MAX_SIZE_MB=128
export DIRECTORY=/folder-to-watch
export INTERVAL_SECONDS=60
export RUST_LOG=info/debug/error/warn
```
