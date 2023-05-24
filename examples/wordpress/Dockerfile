FROM public.ecr.aws/docker/library/wordpress

ENV AWS_LWA_PORT=80

COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.7.0 /lambda-adapter /opt/extensions/lambda-adapter
