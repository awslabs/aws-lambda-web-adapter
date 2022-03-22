FROM public.ecr.aws/docker/library/python:3.8.12-slim-buster
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.3.1 /opt/extensions/lambda-adapter /opt/extensions/lambda-adapter
WORKDIR /var/task
COPY app.py requirements.txt ./
RUN python -m pip install -r requirements.txt
CMD ["gunicorn", "-b=:8080", "-w=1", "app:app"]
