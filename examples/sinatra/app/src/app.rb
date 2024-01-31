require 'sinatra'
# NOTE: logger.info => STDERR or CloudWatch Logs

get '/' do
  logger.info params # QueryString
  'Hello! I am <b>Sinatra</b>.'
end

post '/' do
  logger.info params # QueryString and/or x-www-form-urlencoded
  logger.info request.body.read # e.g. application/json
  'Post data recived.'
end

post '/events' do
  logger.info request.body.read # Non-HTTP trigger's event
  'success.'
end
