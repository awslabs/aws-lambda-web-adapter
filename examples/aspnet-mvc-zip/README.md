# ASP.NET ZIP

This example shows how to deploy an ASP.NET application using the Lambda Web Adapter packaged as a ZIP file.

The `dotnet6` runtime is used.

```yaml
Resources:
  AspNetOnLambdaWebAdapterFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: src/
      Handler: run.sh
      MemorySize: 1024
      Environment:
        Variables:
          AWS_LAMBDA_EXEC_WRAPPER: /opt/bootstrap
          RUST_LOG: info
      Layers:
        - !Sub arn:aws:lambda:${AWS::Region}:753240598075:layer:LambdaAdapterLayerX86:17
      Events:
        Api:
          Type: HttpApi
          Properties:
            Path: /{proxy+}
            Method: ANY
```

A shell script is used as the handler to startup the ASP.NET web application.

```run.sh
#!/bin/bash

./AspNetLambdaZipWebAdapter
```

## Build & Deploy

Make sure .NET 6 is already installed. Run the following commands on a x86_64 machine. 

```shell
sam build 
sam deploy -g
```
