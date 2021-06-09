FROM public.ecr.aws/nginx/nginx:1.19-alpine
WORKDIR "/tmp"
COPY --from=public.ecr.aws/awsguru/lambda-http-ric:latest /opt/bootstrap /opt/bootstrap
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
ENTRYPOINT ["/opt/bootstrap"]
CMD ["nginx", "-g", "daemon off;"]
