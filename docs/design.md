## How does it work?

AWS Lambda Web Adapter supports AWS Lambda functions triggered by Amazon API Gateway Rest API, Http API (v2 event format), and Application Load Balancer.
Lambda Web Adapter converts incoming events to http requests and sends to web application, and converts the http response back to lambda event response.

Lambda Web Adapter is a Lambda Extension (since 0.2.0). When the docker image is running inside AWS Lambda Service, Lambda will automatic start the Adapter and 
the runtime process. When running outside of Lambda, Lambda Web Adapter does not run at all. This allows developers to package their web application 
as a container image and run it on AWS Lambda, AWS Fargate and Amazon EC2 without changing code.

After Lambda Web Adapter launches the application, it will perform readiness check on http://localhost:8080/ every 10ms.
It will start lambda runtime client after receiving 200 response from the application and forward requests to http://localhost:8080.

![lambda-runtime](images/lambda-adapter-runtime.png)
