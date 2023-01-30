FROM public.ecr.aws/awsguru/php:82-2023.1.30.1-x86_64

COPY --from=composer:latest /usr/bin/composer /usr/local/bin/composer

COPY app /var/task/app

RUN ls /var/task/

WORKDIR /var/task/app

RUN composer install --prefer-dist --optimize-autoloader --no-dev --no-interaction \
    && php artisan optimize:clear \
    && php artisan config:cache \
    && php artisan event:cache \
    && php artisan route:cache \
    && php artisan view:cache \
    && php artisan optimize

RUN mv /var/task/app /.out
