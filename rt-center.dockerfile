FROM rust:1.75.0 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN cargo build -p land-cloud-server --release

FROM ubuntu:latest
EXPOSE 3040
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-cloud-server /opt/bin/land-cloud-server
CMD ["./land-cloud-server"]