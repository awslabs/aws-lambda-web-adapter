FROM public.ecr.aws/lambda/nodejs:14
COPY --from=public.ecr.aws/awsguru/lambda-adapter:0.1.0 /opt/bootstrap /opt/bootstrap
EXPOSE 8080
WORKDIR "/var/task"
ADD extensions/ /opt
ADD src/ /var/task
RUN npm install --production
ENTRYPOINT ["/opt/bootstrap"]
CMD ["node", "index.js"]
