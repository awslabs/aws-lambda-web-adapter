#!/bin/sh

php artisan optimize:clear
php artisan config:clear
php artisan event:clear
php artisan route:clear
php artisan view:clear

php-fpm --force-stderr --fpm-config /var/task/php/etc/php-fpm.conf

exec /opt/nginx/bin/nginx -c /var/task/nginx/conf/nginx.conf -g "daemon off;"
