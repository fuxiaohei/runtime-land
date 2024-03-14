FROM rust:1.76.0 as build

WORKDIR /usr/src/land-server
COPY . .
RUN rustup component add rustfmt
RUN cargo build -p land-server --release

FROM debian:stable-slim
WORKDIR /opt/bin/
COPY --from=build /usr/src/land-server/target/release/land-server /opt/bin/land-server
EXPOSE 8840
CMD ["/opt/bin/land-server","--verbose"]

