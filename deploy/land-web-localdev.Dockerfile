FROM node as builder

WORKDIR /usr/src/runtime-land
ADD ./web ./web
WORKDIR /usr/src/runtime-land/web
RUN npm install
RUN npm run build

FROM nginx
EXPOSE 80
WORKDIR /opt/bin/
COPY --from=builder /usr/src/runtime-land/web/dist /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]