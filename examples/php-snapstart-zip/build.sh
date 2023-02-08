#!/bin/sh

composer install --prefer-dist --optimize-autoloader --no-dev --no-interaction

php artisan optimize:clear
php artisan config:cache
php artisan event:cache
php artisan route:cache
php artisan view:cache
php artisan optimize
