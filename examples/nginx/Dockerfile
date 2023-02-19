FROM public.ecr.aws/awsguru/nginx

COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.6.2 /lambda-adapter /opt/extensions/lambda-adapter

# config files
ADD nginx/conf/nginx.conf /opt/nginx/conf/nginx.conf

# code files
COPY app /var/task/

EXPOSE 8080
