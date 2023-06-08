FROM node as builder

WORKDIR /usr/src/moni-serverless
ADD . .
WORKDIR /usr/src/moni-serverless/moni-web
RUN npm install
RUN npm run build

FROM nginx
EXPOSE 80
WORKDIR /opt/bin/
COPY --from=builder /usr/src/moni-serverless/moni-web/dist /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]