#!/bin/bash

args=("$@")

exec -- "/opt/bootstrap" "${args[@]:0:$#-1}" "${LAMBDA_TASK_ROOT}/${_HANDLER}"