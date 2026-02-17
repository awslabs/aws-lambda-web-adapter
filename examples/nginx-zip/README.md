# nginx-zip

This example shows how to use Lambda Adapter to run a Nginx application on AWS Lambda.

### How does it work?

We add Nginx layer to the function and configure wrapper script.

1. attach Nginx layer to your function. This layer contains the Nginx binary and a wrapper script.
    1. x86_64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Nginx123X86:15`
    2. arm64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Nginx123Arm:15`

### Build and Deploy

Run the following commands to build and deploy the application to lambda.

```bash
sam build
sam deploy --guided
```

```shell
...
---------------------------------------------------------------------------------------------------------
OutputKey-Description              OutputValue
---------------------------------------------------------------------------------------------------------
URL - URL for application          https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
---------------------------------------------------------------------------------------------------------
...

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/
```

When the deployment completes, take note of URL's Value. It is the API Gateway endpoint URL.

Lambda Adapter also automatic encode/decode binary data for you. Open the output link in your browser, add "images/space.jpeg" to the url, you will see a picture of the space.

https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/images/space.jpeg

![space](app/public/images/space.jpeg)

### Verify it works

Open URL's URL in a browser, you should see "Nginx" on the page. 
