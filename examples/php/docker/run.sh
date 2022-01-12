#!/bin/bash

/usr/sbin/php-fpm

exec nginx -g "daemon off;";