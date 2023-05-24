FROM rust as builder

WORKDIR /usr/src/moni-serverless
ADD . .
RUN apt update
RUN apt install -y protobuf-compiler libprotobuf-dev
RUN rustup component add rustfmt
RUN cargo build --release

FROM debian:stable-slim
EXPOSE 38779
WORKDIR /opt/bin/
COPY --from=builder /usr/src/moni-serverless/target/release/moni-serverless /opt/bin/moni-serverless
COPY --from=builder /usr/src/moni-serverless/moni-serverless.toml /opt/bin/moni-serverless.toml
CMD ["./moni-serverless"]