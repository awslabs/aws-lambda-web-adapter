<?php

namespace App\Http\Controllers;

use Aws\DynamoDb\DynamoDbClient;
use Aws\Result;
use Ramsey\Uuid\Uuid;

class HomeController extends Controller
{

    public function home()
    {
        return view('welcome');
    }

    public function phpinfo()
    {
        if (!isset($_ENV['HTTP_X_AMZN_REQUEST_CONTEXT'])) {
            return phpinfo();
        }

        $this->handle();

        $request_context = json_decode($_ENV['HTTP_X_AMZN_REQUEST_CONTEXT'], true);


        $requestId            = $request_context['requestId'];
        $time                 = $request_context['time'];
        $time_api_gw          = $request_context['timeEpoch'];
        $time_lambda          = $this->ms($_ENV['REQUEST_TIME_FLOAT']);
        $time_lambda_instance = $this->ms();

        $_ENV['REQUESTID'] = $requestId;
        $_ENV['TIME']      = $time;

        $_ENV['TIME_API_GW']          = $time_api_gw;
        $_ENV['TIME_API_GW_COST']     = $time_lambda - $time_api_gw;
        $_ENV['TIME_LAMBDA']          = $time_lambda;
        $_ENV['TIME_LAMBDA_COST']     = $time_lambda_instance - $time_lambda;
        $_ENV['TIME_LAMBDA_INSTANCE'] = $time_lambda_instance;

        $_ENV['COST_FROM_REQUEST'] = $time_lambda_instance - $time_api_gw;

        return phpinfo();
    }

    public function ms($string = null): string
    {
        if ($string === null) {
            $string = (string)microtime(true);
        }

        $string = str_replace(".", "", $string);

        $string .= "0000";

        return mb_strcut($string, 0, 13);
    }

    /**
     * @param $region
     * @param $table
     * @param $data
     * @return Result
     */
    protected function batchWriteItem($region, $table, $data): Result
    {
        $client = new DynamoDbClient([
            'version' => '2012-08-10',
            'region' => $region,
        ]);

        return $client->batchWriteItem([
            'RequestItems' => [
                $table => $data,
            ],
        ]);
    }

    /**
     * Execute the console command.
     *
     * @return int
     */
    public function handle(): int
    {

        $events = [];

        $count = 0;

        $region = 'us-west-2';
        $table  = 'prod-lambda-runtimes-tests-cost';

        for ($i = 0; $i < 1; $i++) {
            $count++;
            $item = [
                'PutRequest' => [
                    'Item' => [
                        'id' => ['S' => Uuid::uuid4()],
                        'request_at_ms' => ['S' => $_GET['request_at_ms'] ?? $this->ms()],
                        'php_start_at' => ['S' => $this->ms($_ENV['REQUEST_TIME_FLOAT'])],
                        'php_started_at' => ['S' => $this->ms()],
                        'aws_execution_env' => ['S' => $_ENV['AWS_EXECUTION_ENV']],
                        'env' => ['S' => json_encode($_ENV)],
                    ],
                ],
            ];

            $events[] = $item;

            if (count($events) === 10) {
                $this->batchWriteItem($region, $table, $events);
                $events = [];
            }
        }

        if (count($events)) {
            $this->batchWriteItem($region, $table, $events);
        }

//			dump("$count items was written in $region.");

        return 0;
    }

}
