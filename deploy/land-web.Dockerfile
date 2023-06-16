FROM node:20.3.0 as builder
WORKDIR /usr/src/runtime.land
ADD ./web /usr/src/runtime.land/web
WORKDIR /usr/src/runtime.land/web
RUN npm install
RUN npm run build

FROM nginx:1.25
EXPOSE 80
WORKDIR /opt/bin/
ADD web/run-in-docker.sh /opt/bin/run-in-docker.sh
COPY --from=builder /usr/src/runtime.land/web/dist /usr/share/nginx/html
COPY --from=builder /usr/src/runtime.land/web/land-web.nginx.conf /etc/nginx/conf.d/default.conf
CMD ["./run-in-docker.sh"]