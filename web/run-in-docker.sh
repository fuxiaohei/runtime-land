#!/usr/bin/env bash

set -e
set -o pipefail

echo "Set api config.js by env"
echo "window.API_ADDRESS='$API_ADDRESS';" >> /usr/share/nginx/html/config.js

echo "Current config.js:"
cat /usr/share/nginx/html/config.js;

echo "Start nginx:"
nginx -g 'daemon off;'