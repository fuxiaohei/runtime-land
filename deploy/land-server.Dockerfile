FROM ubuntu:latest
EXPOSE 38779
WORKDIR /opt/bin/
COPY target/release/lol-server /opt/bin/lol-server
CMD ["./lol-server"]