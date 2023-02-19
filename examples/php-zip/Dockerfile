FROM public.ecr.aws/awsguru/php AS builder

COPY --from=composer:latest /usr/bin/composer /usr/local/bin/composer

COPY app /var/task/app

WORKDIR /var/task/app

RUN composer install --prefer-dist --optimize-autoloader --no-dev --no-interaction

RUN mv /var/task/app /.out
