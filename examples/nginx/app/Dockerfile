FROM public.ecr.aws/nginx/nginx:1.19-alpine
EXPOSE 8080
WORKDIR "/tmp"
COPY --from=public.ecr.aws/awsguru/lambda-http-ric:latest /opt/bootstrap /opt/bootstrap
ADD config/ /etc/nginx/
ENTRYPOINT ["/opt/bootstrap"]
CMD ["nginx", "-g", "daemon off;"]
