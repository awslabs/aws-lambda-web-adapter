FROM public.ecr.aws/lambda/nodejs:14
COPY --from=public.ecr.aws/awsguru/lambda-http-ric:latest /opt/bootstrap /opt/bootstrap
EXPOSE 8080
WORKDIR "/var/task"
ADD src/ /var/task
RUN npm install
ENTRYPOINT ["/opt/bootstrap"]
CMD ["node", "index.js"]
