FROM mccutchen/go-httpbin:v2.4.1
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.4.1 /lambda-adapter /opt/extensions/lambda-adapter
CMD ["/bin/go-httpbin"]