#!/bin/sh

if [ ! -d '/tmp/session' ]; then
  mkdir -p /tmp/session
fi

/opt/php/bin/php-fpm --force-stderr --fpm-config /var/task/php/etc/php-fpm.conf

exec /opt/nginx/bin/nginx -c /var/task/nginx/conf/nginx.conf -g "daemon off;"
