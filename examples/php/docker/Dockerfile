FROM public.ecr.aws/amazonlinux/amazonlinux:2.0.20211001.1

COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.1.0 /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]

RUN amazon-linux-extras install -y nginx1 php7.4

# Nginx config
COPY nginx/ /etc/nginx/

# PHP config
COPY php/ /etc/

# Expose ports
EXPOSE 8080

# Define default command.
COPY run.sh /run.sh
CMD ["/run.sh"]
