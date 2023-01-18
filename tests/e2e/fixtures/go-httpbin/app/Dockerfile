FROM mccutchen/go-httpbin:v2.4.1
COPY --from=477159140107.dkr.ecr.ap-northeast-1.amazonaws.com/awsguru/aws-lambda-adapter:latest /lambda-adapter /opt/extensions/lambda-adapter
CMD ["/bin/go-httpbin"]