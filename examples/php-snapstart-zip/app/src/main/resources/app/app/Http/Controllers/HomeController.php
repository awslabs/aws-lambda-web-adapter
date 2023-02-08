<?php

namespace App\Http\Controllers;

use Aws\DynamoDb\DynamoDbClient;
use Aws\Result;

class HomeController extends Controller
{

    public function home()
    {
        return view('welcome');
    }

    public function phpinfo()
    {
        $time_epoch_app        = $this->ms(LARAVEL_START);
        $time_epoch_controller = $this->ms();

        $_ENV['MEMORY_GET_USAGE'] = memory_get_usage();

        if (!isset($_ENV['HTTP_X_AMZN_REQUEST_CONTEXT'])) {
            return phpinfo();
        }

        $request_context = json_decode($_ENV['HTTP_X_AMZN_REQUEST_CONTEXT'], true);


        $requestId = $request_context['requestId'];
        $time      = $request_context['time'];

        $time_epoch_api_gw = $this->ms($request_context['timeEpoch']);
        $time_epoch_lambda = $this->ms($_ENV['REQUEST_TIME_FLOAT']);


        $_ENV['REQUESTID'] = $requestId;
        $_ENV['TIME']      = $time;

        $_ENV['TIME_EPOCH_API_GW']      = $time_epoch_api_gw;
        $_ENV['TIME_API_GW_DURATION']   = $time_epoch_lambda - $time_epoch_api_gw;
        $_ENV['TIME_EPOCH_LAMBDA']      = $time_epoch_lambda;
        $_ENV['TIME_LAMBDA_DURATION']   = $time_epoch_app - $time_epoch_lambda;
        $_ENV['TIME_EPOCH_APP']         = $time_epoch_app;
        $_ENV['TIME_APP_INIT_DURATION'] = $time_epoch_controller - $time_epoch_app;
        $_ENV['TIME_EPOCH_CONTROLLER']  = $time_epoch_controller;

        $_ENV['DURATION_FROM_API_GW'] = $time_epoch_controller - $time_epoch_api_gw;

        $events = [];

        $events[] = [
            'PutRequest' => [
                'Item' => [
                    'id' => ['S' => $_ENV['REQUESTID']],
                    'time' => ['S' => $_ENV['TIME']],

                    'time_epoch_api_gw' => ['N' => $_ENV['TIME_EPOCH_API_GW']],
                    'time_api_gw_duration' => ['N' => $_ENV['TIME_API_GW_DURATION']],
                    'time_epoch_lambda' => ['N' => $_ENV['TIME_EPOCH_LAMBDA']],
                    'time_lambda_duration' => ['N' => $_ENV['TIME_LAMBDA_DURATION']],
                    'time_epoch_app' => ['N' => $_ENV['TIME_EPOCH_APP']],
                    'time_app_init_duration' => ['N' => $_ENV['TIME_APP_INIT_DURATION']],
                    'time_epoch_controller' => ['N' => $_ENV['TIME_EPOCH_CONTROLLER']],
                    'duration_from_api_gw' => ['N' => $_ENV['DURATION_FROM_API_GW']],

                    'memory_get_usage' => ['N' => $_ENV['MEMORY_GET_USAGE']],

                    'aws_execution_env' => ['S' => $_ENV['AWS_EXECUTION_ENV']],
                    'version' => ['S' => PHP_VERSION],
                    'env' => ['S' => json_encode($_ENV)],
                ],
            ],
        ];

        $this->batchWriteItem('us-west-2', 'prod-lambda-runtimes-tests-cost', $events);

        return phpinfo();
    }

    public function ms($string = null): int
    {
        if ($string === null) {
            $string = (string)microtime(true);
        }

        $string = str_replace(".", "", $string);

        $string .= "0000";

        return (int)mb_strcut($string, 0, 13);
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

}
