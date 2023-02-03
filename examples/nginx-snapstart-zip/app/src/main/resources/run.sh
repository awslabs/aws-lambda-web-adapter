#!/bin/sh

exec /opt/nginx/bin/nginx -c /var/task/nginx/conf/nginx.conf -g "daemon off;"
