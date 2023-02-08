FROM public.ecr.aws/awsguru/php

COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.6.1 /lambda-adapter /opt/extensions/lambda-adapter

COPY app/src/main/resources/app /var/task/app
WORKDIR /var/task/app

# config files
ADD app/src/main/resources/nginx/conf/nginx.conf      /opt/nginx/conf/nginx.conf
ADD app/src/main/resources/php/php.ini                /opt/php/php.ini
ADD app/src/main/resources/php/etc/php-fpm.conf       /opt/php/etc/php-fpm.conf
ADD app/src/main/resources/php/php.d/extensions.ini   /opt/php/php.d/extensions.ini
