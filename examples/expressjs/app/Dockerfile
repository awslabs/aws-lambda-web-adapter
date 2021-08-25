FROM public.ecr.aws/lambda/nodejs:14
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
EXPOSE 8080
WORKDIR "/var/task"
ADD extensions/ /opt
ADD src/package.json /var/task/package.json
ADD src/package-lock.json /var/task/package-lock.json
RUN npm install --production
ADD src/ /var/task
CMD ["node", "index.js"]
