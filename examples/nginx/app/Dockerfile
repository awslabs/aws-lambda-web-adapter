FROM public.ecr.aws/docker/library/nginx:1.21.6
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.3.1 /opt/extensions/lambda-adapter /opt/extensions/lambda-adapter
WORKDIR "/tmp"
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
CMD ["nginx", "-g", "daemon off;"]
