#!/bin/bash

PATH=$PATH:$LAMBDA_TASK_ROOT/bin PYTHONPATH=$LAMBDA_TASK_ROOT exec gunicorn -b=:$PORT -w=1 app:app
