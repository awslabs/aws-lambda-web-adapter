FROM public.ecr.aws/nginx/nginx:1.21.1
WORKDIR "/tmp"
COPY --from=public.ecr.aws/awsguru/lambda-adapter:0.1.0 /opt/bootstrap /opt/bootstrap
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
ADD extensions/ /opt
ENTRYPOINT ["/opt/bootstrap"]
CMD ["nginx", "-g", "daemon off;"]
