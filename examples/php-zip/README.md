# php-zip

This example shows how to use PHP Runtime to run a PHP application on managed PHP runtime.

### How does it work?

We add PHP Runtime layer to the function and configure wrapper script.

1. attach PHP Runtime layer to your function. This layer containers PHP Runtime binary and a wrapper script.
    1. x86_64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Php82FpmNginxX86:8`
    2. arm64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Php82FpmNginxArm:8`
2. configure Lambda environment variable `AWS_LAMBDA_EXEC_WRAPPER` to `/opt/bootstrap`. This is a wrapper script
   included in the layer.

To get more information of Wrapper script, please read Lambda
documentation [here](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-modify.html#runtime-wrapper).

### Build and Deploy

Run the following commands to build and deploy the application to lambda.

```bash
sam build
sam deploy --guided
```

When the deployment completes, take note of URL's Value. It is the API Gateway endpoint URL.

### Verify it works

Open URL's URL in a browser, you should see `phpinfo()` on the page. 
