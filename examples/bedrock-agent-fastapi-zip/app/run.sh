#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin:/opt/bin PYTHONPATH=/opt/python:$LAMBDA_RUNTIME_DIR exec python -m uvicorn --port=$PORT main:app
