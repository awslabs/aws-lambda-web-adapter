#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin:/opt/bin PYTHONPATH=$LAMBDA_RUNTIME_DIR:/opt/python exec python -m uvicorn --port=$PORT main:app
