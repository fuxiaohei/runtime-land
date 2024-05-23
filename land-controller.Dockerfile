FROM rust:1.78 as build

WORKDIR /usr/src/land-controller
COPY . .
RUN rustup component add rustfmt
RUN bash /usr/src/land-controller/deploy/deps.sh
RUN cargo version
RUN cargo build -p land-controller --release

FROM debian:stable-slim
WORKDIR /opt/bin/
RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean
COPY --from=build /usr/src/land-controller/target/release/land-controller /opt/bin/land-controller
COPY --from=build /usr/src/land-controller/wizer-v6.0.0-x86_64-linux /opt/bin/wizer
EXPOSE 9840
CMD ["/opt/bin/land-controller","--verbose"]

