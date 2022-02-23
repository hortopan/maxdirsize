FROM rust

WORKDIR /app

COPY Cargo.lock /app/
COPY Cargo.toml /app/

# Hack to make Cargo download and cache dependencies
RUN \
    mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src /app/src

RUN \
    touch src/main.rs && \
    cargo test --release && \
    cargo build --release && \
    mv target/release/maxdirsize bin && \
    rm -rf target

FROM ubuntu:hirsute

COPY --from=0 /app/bin /app

ENTRYPOINT ["/app"]
