FROM public.ecr.aws/amazonlinux/amazonlinux:2.0.20220121.0

COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.3.1 /opt/extensions/lambda-adapter /opt/extensions/lambda-adapter

RUN amazon-linux-extras install -y nginx1 php7.4 && yum clean all && rm -rf /var/cache/yum

# Nginx config
COPY nginx/ /etc/nginx/

# PHP config
COPY php/ /etc/

# Expose ports
EXPOSE 8080

# Define default command.
COPY run.sh /run.sh
CMD ["/run.sh"]
