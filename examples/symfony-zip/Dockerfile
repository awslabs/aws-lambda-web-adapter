FROM public.ecr.aws/awsguru/php:devel-82-2023.1.30.1

COPY --from=composer:latest /usr/bin/composer /usr/local/bin/composer

COPY app /var/task/app

WORKDIR /var/task/app

RUN composer install --prefer-dist --optimize-autoloader --no-interaction \
        && ./bin/console cache:warmup

RUN mv /var/task/app /.out
