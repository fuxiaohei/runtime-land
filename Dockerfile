FROM rust:1.71.1 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN cargo --version --verbose
RUN rustc --version
RUN cargo build --release -p land-center

FROM ubuntu:latest
RUN apt update && apt install -y ca-certificates && update-ca-certificates
EXPOSE 7901
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-center /opt/bin/land-center
CMD ["./land-center"]