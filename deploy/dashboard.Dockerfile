FROM node:20.3.0 as builder
WORKDIR /usr/src/runtimm-land
ADD ./web /usr/src/runtime-land/web
WORKDIR /usr/src/runtime-land/web
RUN npm install
RUN npm run build

FROM nginx:1.25
EXPOSE 80
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/web/dist /usr/share/nginx/html
COPY --from=builder /usr/src/runtime-land/web/web-nginx.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]
