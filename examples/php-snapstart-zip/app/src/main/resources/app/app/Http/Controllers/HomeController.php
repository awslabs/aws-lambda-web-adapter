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

        $current = (string)microtime(true);
        $current = str_replace(".", "", $current);
        $current .= "0000";
        $ms      = mb_strcut($current, 0, 13);

        $request_context = json_decode($_ENV['HTTP_X_AMZN_REQUEST_CONTEXT'], true);

        $_ENV['TEST_REQUESTID'] = $request_context['requestId'];
        $_ENV['TEST_TIME']      = $request_context['time'];
        $_ENV['TEST_TIMEEPOCH'] = $request_context['timeEpoch'];
        $_ENV['TEST_MS']        = $ms;

        return phpinfo();
    }

}
