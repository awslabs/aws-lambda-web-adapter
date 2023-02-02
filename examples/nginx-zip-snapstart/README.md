# Nginx in SnapStart example

A basic pet store application written with the Nginx. You can build and test it locally as a typical Nginx web application.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). The `template.yaml` file in the root folder contains the application definition.

To run the application we are using the run.sh script located in the resources folder:

```shell
#!/bin/sh

exec java -cp "./:lib/*" "com.amazonaws.demo.petstore.Application"
```

In the configuration we have to specify the AWS Lambda adapter as a layer and configure the script as handler:

```yaml
 Properties:
      MemorySize: 512
      Handler: run.sh
      CodeUri: app/
      Runtime: java11
      AutoPublishAlias: live
      SnapStart:
        ApplyOn: PublishedVersions      
      Environment:
        Variables:
          RUST_LOG: info
          READINESS_CHECK_PATH: /healthz
          REMOVE_BASE_PATH: /v1
          AWS_LAMBDA_EXEC_WRAPPER: /opt/bootstrap
      Layers:
        - !Sub arn:aws:lambda:${AWS::Region}:753240598075:layer:LambdaAdapterLayerX86:11
```
In this template, we enable SnapStart for this function. SnapStart drastically reduces cold start time for Java functions using Firecracker MicroVM snapshotting technology. Read more about SnapStart [here](https://docs.aws.amazon.com/lambda/latest/dg/snapstart.html).

### Remove the base path

The pet store application is deployed under /v1/{proxy+}. But the application does not know that. So in the SAM template file, we configured environment variable `REMOVE_BASE_PATH=/v1`. 
This configuration tells the Adapter to remove `/v1` from http request path, so that the pet store application works without changing code. 


## Pre-requisites

The following tools should be installed and configured.

* [AWS CLI](https://aws.amazon.com/cli/)
* [SAM CLI](https://github.com/awslabs/aws-sam-cli)
* [Maven](https://maven.apache.org/)
* [Docker](https://www.docker.com/products/docker-desktop)

## Deploy to Lambda
Navigate to the sample's folder and use the SAM CLI to build the application:

```shell
$ sam build
```

This command compiles the application and prepares a deployment package in the `.aws-sam` sub-directory.

To deploy the application in your AWS account, you can use the SAM CLI's guided deployment process and follow the instructions on the screen

```shell
$ sam deploy --guided
```

Once the deployment is completed, the SAM CLI will print out the stack's outputs, including the new application URL. You can use `curl` or a web browser to make a call to the URL

```shell
...
---------------------------------------------------------------------------------------------------------
OutputKey-Description                        OutputValue
---------------------------------------------------------------------------------------------------------
PetStoreApi - URL for application            https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/v1/pets
---------------------------------------------------------------------------------------------------------
...

$ curl https://xxxxxxxxxx.execute-api.us-west-2.amazonaws.com/v1/pets
```

## Clean up

This example use provisioned concurrency to reduce cold start time. It incurs additional cost. You can remove the whole example with the following command. 

```shell
sam delete
```
