FROM rust:1.70.0 as builder
WORKDIR /usr/src/runtime.land
ADD . .
RUN apt update && apt install -y protobuf-compiler libprotobuf-dev
RUN cargo --version --verbose
RUN rustc --version
RUN protoc --version
RUN make build-server

FROM ubuntu:latest
EXPOSE 38779
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime.land/target/release/land-server /opt/bin/land-server
CMD ["./land-server"]