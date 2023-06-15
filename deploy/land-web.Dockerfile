FROM node as builder
WORKDIR /usr/src/runtime.land
ADD . .
WORKDIR /usr/src/runtime.land/web
RUN npm install
RUN npm run build

FROM nginx
EXPOSE 80
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime.land/web/dist /usr/share/nginx/html
COPY --from=builder /usr/src/runtime.land/deploy/nginx.default.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]