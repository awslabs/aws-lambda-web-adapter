<?php

namespace App\Http\Controllers;

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

}
