FROM rust:1.71.0 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN cargo --version --verbose
RUN rustc --version
RUN cargo build --release -p land-center

FROM ubuntu:latest
EXPOSE 7899
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-center /opt/bin/land-center
CMD ["./land-center"]