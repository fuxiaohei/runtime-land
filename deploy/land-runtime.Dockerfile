FROM ubuntu:latest
EXPOSE 38889
WORKDIR /opt/bin/
COPY target/release/land-runtime /opt/bin/land-runtime
CMD ["./land-runtime"]