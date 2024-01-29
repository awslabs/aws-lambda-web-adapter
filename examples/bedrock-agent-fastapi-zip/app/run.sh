#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin:/opt/bin PYTHONPATH=$LAMBDA_TASK_ROOT:/opt/python exec python -m uvicorn --port=$PORT main:app
