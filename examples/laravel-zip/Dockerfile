FROM public.ecr.aws/awsguru/php:82.2023.3.13.1

COPY --from=composer:latest /usr/bin/composer /usr/local/bin/composer

COPY app /var/task/app

WORKDIR /var/task/app

RUN composer install --prefer-dist --optimize-autoloader --no-dev --no-interaction \
    && php artisan optimize:clear \
    && php artisan config:cache \
    && php artisan event:cache \
    && php artisan route:cache \
    && php artisan view:cache \
    && php artisan optimize
