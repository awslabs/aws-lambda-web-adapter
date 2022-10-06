#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin PYTHONPATH=$LAMBDA_TASK_ROOT exec uvicorn --port=$PORT main:app
