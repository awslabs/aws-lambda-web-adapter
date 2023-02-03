#!/bin/sh

if [ ! -d '/tmp/session' ]; then
  mkdir -p /tmp/session
fi

php artisan optimize:clear
php artisan config:clear
php artisan event:clear
php artisan route:clear
php artisan view:clear

php-fpm --force-stderr --fpm-config /var/task/php/etc/php-fpm.conf

exec nginx -c /var/task/nginx/conf/nginx.conf -g "daemon off;"
