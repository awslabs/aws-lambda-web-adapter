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

        $_ENV['TEST_REQUESTID']    = $request_context['requestId'];
        $_ENV['TEST_TIME']         = $request_context['time'];
        $_ENV['TEST_TIMEEPOCH']    = $request_context['timeEpoch'];
        $_ENV['TEST_REQUEST_TIME'] = $this->ms($_ENV['REQUEST_TIME_FLOAT']);
        $_ENV['TEST_MS']           = $this->ms();
        $_ENV['TEST_COST']         = $this->ms() - $_ENV['TEST_TIMEEPOCH'];

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
