FROM public.ecr.aws/runecast/nginx:1.21.1-alpine
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
WORKDIR "/tmp"
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
CMD ["nginx", "-g", "daemon off;"]
