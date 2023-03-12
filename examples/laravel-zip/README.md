# laravel-zip

This example shows how to run a Laravel application on AWS Lambda.

### How does it work?

We add PHP layer to the function and configure wrapper script.

1. attach PHP layer to your function. This layer containers PHP binary and a wrapper script.
    1. x86_64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Php82FpmNginxX86:12`
    2. arm64: `arn:aws:lambda:${AWS::Region}:753240598075:layer:Php82FpmNginxArm:12`

## Laravel Configuration

Set up your `.env` file.

```shell
$ cd app
$ cp .env.example .env
$ composer install --prefer-dist --optimize-autoloader --no-interaction
$ php artisan key:generate
```

Use S3 as Filesystem:

```dotenv
FILESYSTEM_DISK=s3
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
AWS_DEFAULT_REGION=
AWS_BUCKET=
```

Use stdout as Log:
```dotenv
LOG_CHANNEL=single
```

Edit `config/logging.php` -> `single`
```php
'path' => 'php://stdout',
```

Use `redis` as Cache and Session driver:
```dotenv
CACHE_DRIVER=redis
SESSION_DRIVER=redis
REDIS_HOST=
REDIS_PASSWORD=
REDIS_PORT=6379
```

### Build and Deploy

Run the following commands to build and deploy the application to lambda.

```bash
sam build
sam deploy --guided
```

When the deployment completes, take note of URL's Value. It is the API Gateway endpoint URL.

### Verify it works

Open URL's URL in a browser, you should see `laravel` on the page. 
