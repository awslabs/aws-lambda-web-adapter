FROM public.ecr.aws/lambda/python:3.8
COPY --from=aws-lambda-adapter:latest /opt/bootstrap /opt/bootstrap
ENTRYPOINT ["/opt/bootstrap"]
WORKDIR /var/task
COPY app.py requirements.txt ./
RUN python3.8 -m pip install -r requirements.txt
CMD ["gunicorn", "-b=:8080", "-w=1", "app:app"]
