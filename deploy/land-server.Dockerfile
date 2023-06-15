FROM ubuntu:latest
EXPOSE 38779
WORKDIR /opt/bin/
COPY target/release/land-server /opt/bin/land-server
CMD ["./land-server"]