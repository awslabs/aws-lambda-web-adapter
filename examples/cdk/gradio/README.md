# Gradio example

A basic Gradio application example. You can build and test it locally as a typical Gradio application.

Using AWS Lambda Web Adapter, You can package this web application into Docker image, push to ECR, and deploy to Lambda, ECS/EKS, or EC2.

The application can be deployed in an AWS account using the [Serverless Application Model](https://github.com/awslabs/serverless-application-model). 


Here will see how to do it by [AWS CDK](https://aws.amazon.com/cdk/) 

# Pre-requisites
* [AWS CLI](https://aws.amazon.com/cli/)
* [Python](https://www.python.org/)
* [Docker](https://www.docker.com/products/docker-desktop)
* [AWS CDK](https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html)



### IAM Rules:
- permission to Pull images from Public Repository
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": "ecr-public:PutRegistryAlias",
            "Resource": "arn:aws:ecr-public::123456789012:registry/*"
        }
    ]
}
```

### CDK Env variables
```bash
CDK_DEFAULT_ACCOUNT=$(aws sts get-caller-identity --query Account)
CDK_DEFAULT_REGION=$(aws configure get region)
```



## Dockerfile

```Dockerfile
# Base Image
FROM public.ecr.aws/docker/library/python:3.8.12-slim-buster 
# Lambda Adapter
# lambda adapter binary into /opt/extensions. This is the only change to run the application on Lambda.
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.4 /lambda-adapter /opt/extensions/lambda-adapter
ENV PORT=8080

WORKDIR /var/task

# Install system dependencies if needed
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*

# Copy and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Expose Gradio default port
EXPOSE 8080

# Use explicit Python interpreter path
CMD ["python", "app.py"]
```

Line 2 *copies lambda adapter binary into /opt/extensions. This is the only change to run the Flask application on Lambda.*

# Build and Deploy

In cdk.json we focusing on cdk.py
```json
{
    "app": "python3 cdk.py"
}
```


In addition to Lambda function add function url so it'll easy for redirect. but it'll refresh whenever lambda function refreshes.

```py
# STACK
class GradioLambdaFn(Stack):
    def __init__(self,scope:Construct,construct_id:str,**kwargs)->None:
        super().__init__(scope,construct_id,**kwargs)
        
        # Lambda Fn
        lambda_fn = DockerImageFunction(
                            self,
                            id="AWSLambdaAdapterGradioExample",
                            code=DockerImageCode.from_image_asset( directory= os.path.dirname(__file__) , file="Dockerfile"),
                            architecture=Architecture.X86_64,
                            timeout=Duration.minutes(10),                   
        )

        # HTTP URL add
        fn_url = lambda_fn.add_function_url(auth_type=FunctionUrlAuthType.NONE)

        # print
        CfnOutput(self,id="functionURL",value=fn_url.url,description="cfn_output")

# APP
app = App()
gradio_lambda = GradioLambdaFn(app,"GradioLambda",env=env)
app.synth()
```



# Deploy CDK
Navigate to folder and use the CDK CLI to build a CloudFormation Stack

To deploy cdk app into AWS
```bash
cdk synth 
cdk deploy --verbose
cdk destroy
```

## Verify it works
Open URL's URL in a browser, here we print in the console by `CfnOutput` you should see "Gradio" on the page. 