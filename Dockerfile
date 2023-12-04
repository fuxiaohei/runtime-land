FROM rust:1.73.0 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN cargo --version --verbose
RUN rustc --version
RUN ./deploy/build-center.sh

FROM ubuntu:latest
RUN apt update && apt install -y ca-certificates && update-ca-certificates
EXPOSE 7901
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-center /opt/bin/land-center
COPY --from=builder /usr/src/runtime-land/templates-wasm.tar.gz /opt/bin/templates-wasm.tar.gz
RUN tar -xvf templates-wasm.tar.gz
CMD ["./land-center"]