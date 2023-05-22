FROM rust as builder

WORKDIR /usr/src/moni-serverless
ADD . .
RUN rustup component add rustfmt
RUN cargo build --release

FROM debian:stable-slim
EXPOSE 38779
WORKDIR /opt/bin/
COPY --from=builder /usr/src/moni-serverless/target/release/moni-serverless /opt/bin/moni-serverless
CMD ["./moni-serverless"]