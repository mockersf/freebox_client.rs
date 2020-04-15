FROM messense/rust-musl-cross:armv7-musleabihf AS build

RUN mkdir -p /home/rust/src
WORKDIR /home/rust/src
COPY . /home/rust/src

RUN cargo build --release
RUN musl-strip target/armv7-unknown-linux-musleabihf/release/freebox_stats
RUN musl-strip target/armv7-unknown-linux-musleabihf/release/freebox_controller

FROM busybox:musl

COPY --from=build /home/rust/src/target/armv7-unknown-linux-musleabihf/release/freebox_stats /freebin/freebox_stats
COPY --from=build /home/rust/src/target/armv7-unknown-linux-musleabihf/release/freebox_controller /freebin/freebox_controller
