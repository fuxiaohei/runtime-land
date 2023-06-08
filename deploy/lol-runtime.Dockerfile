FROM ubuntu:latest
EXPOSE 38889
WORKDIR /opt/bin/
COPY target/release/lol-runtime /opt/bin/lol-runtime
CMD ["./lol-runtime"]