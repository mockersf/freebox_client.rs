FROM clux/muslrust AS build

RUN mkdir -p /src
WORKDIR /src
COPY . /src

RUN cargo build --release
RUN strip target/x86_64-unknown-linux-musl/release/stats

FROM busybox:musl

COPY --from=build /src/target/x86_64-unknown-linux-musl/release/stats /freebin/stats
