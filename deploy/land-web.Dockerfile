FROM node as builder

WORKDIR /usr/src/lol-serverless
ADD . .
WORKDIR /usr/src/lol-serverless/web
RUN npm install
RUN npm run build

FROM nginx
EXPOSE 80
WORKDIR /opt/bin/
COPY --from=builder /usr/src/lol-serverless/web/dist /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]