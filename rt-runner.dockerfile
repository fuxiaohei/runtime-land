FROM rust:1.75.0 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN cargo build -p land-runner --release

FROM ubuntu:latest
EXPOSE 3040
EXPOSE 3041
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-runner /opt/bin/land-runner
CMD ["./land-runner"]