FROM debian:stable-slim
EXPOSE 38779
WORKDIR /opt/bin/
COPY target/release/moni-serverless /opt/bin/moni-serverless
CMD ["./moni-serverless"]