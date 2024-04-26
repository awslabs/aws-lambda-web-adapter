#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin \
    PYTHONPATH=$LAMBDA_RUNTIME_DIR:$PYTHONPATH:/opt/python \
    exec python -m uvicorn --port=$PORT main:app
