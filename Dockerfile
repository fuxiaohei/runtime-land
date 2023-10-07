FROM --platform=linux/amd64 rust:1.73.0 as builder
WORKDIR /usr/src/runtime-land
ADD . .
RUN rustup component add rustfmt
RUN cargo --version --verbose
RUN rustc --version
RUN cargo build --release -p land-center
RUN cargo build --release -p land-cli

FROM --platform=linux/amd64 rust:1.73.0 as template-builder
WORKDIR /opt/bin/
RUN rustup component add rustfmt
RUN rustup target add wasm32-wasi
ADD deploy/build-templates-wasm.sh build-templates-wasm.sh
COPY --from=builder /usr/src/runtime-land/target/release/land-cli /opt/bin/land-cli
RUN chmod +x build-templates-wasm.sh
RUN ./build-templates-wasm.sh

FROM --platform=linux/amd64 ubuntu:latest
RUN apt update && apt install -y ca-certificates && update-ca-certificates
EXPOSE 7901
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/target/release/land-center /opt/bin/land-center
COPY --from=template-builder /opt/bin/land-cli-nightly-templates-wasm.tar.gz /opt/bin/land-cli-nightly-templates-wasm.tar.gz
RUN tar -xvf land-cli-nightly-templates-wasm.tar.gz -C templates
CMD ["./land-center"]