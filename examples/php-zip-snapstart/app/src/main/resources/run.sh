#!/bin/sh

if [ ! -d '/tmp/session' ]; then
  mkdir -p /tmp/session
fi

# list runtime libs
dirs=$(echo "$LD_LIBRARY_PATH" | tr ":" "\n")
opt_lib=/opt/lib
dirs=${dirs[@]/$opt_lib/}
for dir in $dirs; do
  ls $dir/*so*
done

/opt/php/bin/php-fpm --force-stderr --fpm-config /var/task/php/etc/php-fpm.conf

exec /opt/nginx/bin/nginx -c /var/task/nginx/conf/nginx.conf -g "daemon off;"
