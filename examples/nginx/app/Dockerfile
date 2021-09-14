FROM public.ecr.aws/runecast/nginx:1.21.1-alpine
WORKDIR "/tmp"
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ADD config/ /etc/nginx/
ADD images/ /usr/share/nginx/html/images
ENTRYPOINT ["/opt/bootstrap"]
CMD ["nginx", "-g", "daemon off;"]
