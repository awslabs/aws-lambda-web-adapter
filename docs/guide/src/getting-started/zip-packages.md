# Zip Packages

AWS Lambda Web Adapter works with AWS managed Lambda runtimes via Lambda Layers.

## Setup Steps

### 1. Attach the Lambda Web Adapter Layer

#### AWS Commercial Regions

| Architecture | Layer ARN |
|-------------|-----------|
| x86_64 | `arn:aws:lambda:${AWS::Region}:753240598075:layer:LambdaAdapterLayerX86:26` |
| arm64 | `arn:aws:lambda:${AWS::Region}:753240598075:layer:LambdaAdapterLayerArm64:26` |

#### AWS China Regions

| Region | Architecture | Layer ARN |
|--------|-------------|-----------|
| cn-north-1 (Beijing) | x86_64 | `arn:aws-cn:lambda:cn-north-1:041581134020:layer:LambdaAdapterLayerX86:26` |
| cn-northwest-1 (Ningxia) | x86_64 | `arn:aws-cn:lambda:cn-northwest-1:069767869989:layer:LambdaAdapterLayerX86:26` |

### 2. Set the Exec Wrapper

Configure the Lambda environment variable:

```
AWS_LAMBDA_EXEC_WRAPPER=/opt/bootstrap
```

### 3. Set the Function Handler

Set your function handler to your web application's startup script, e.g. `run.sh`.

## SAM Template Example

```yaml
Resources:
  MyFunction:
    Type: AWS::Serverless::Function
    Properties:
      Runtime: nodejs20.x
      Handler: run.sh
      Layers:
        - !Sub arn:aws:lambda:${AWS::Region}:753240598075:layer:LambdaAdapterLayerX86:26
      Environment:
        Variables:
          AWS_LAMBDA_EXEC_WRAPPER: /opt/bootstrap
          AWS_LWA_PORT: 7000
```

For a complete working example, see the [Express.js in Zip](https://github.com/awslabs/aws-lambda-web-adapter/tree/main/examples/expressjs-zip) example.
