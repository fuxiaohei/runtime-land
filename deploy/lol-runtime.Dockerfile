FROM ubuntu:latest
EXPOSE 38889
WORKDIR /opt/bin/
COPY target/release/moni-runtime /opt/bin/moni-runtime
CMD ["./moni-runtime"]