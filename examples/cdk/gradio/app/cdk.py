import os 
from constructs import Construct 
from aws_cdk import App, Stack,Duration,CfnOutput,Environment
from aws_cdk.aws_lambda import (
    DockerImageFunction,
    DockerImageCode,
    Architecture,
    FunctionUrlAuthType
)


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
                            description="dockerfile_lambda_deploy_using_cdk",
        )

        # HTTP URL add
        fn_url = lambda_fn.add_function_url(auth_type=FunctionUrlAuthType.NONE)

        # print
        CfnOutput(self,id="functionURL",value=fn_url.url,description="cfn_output")


# My Environment
env = Environment(
    account= os.environ.get('CDK_DEFAULT_ACCOUNT'),
    region=os.environ.get('CDK_DEFAULT_REGION')
)

app = App()
gradio_lambda = GradioLambdaFn(app,"GradioLambda",env=env)
app.synth()