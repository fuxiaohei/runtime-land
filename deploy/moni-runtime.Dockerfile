FROM debian:stable-slim
EXPOSE 38779
WORKDIR /opt/bin/
COPY target/release/moni-runtime /opt/bin/moni-runtime
CMD ["./moni-runtime"]