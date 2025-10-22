import { endpoint } from './config'
import { get_token } from './auth'


function as_url(path) {
  let url = endpoint + path;
  return url;
}

function headers() {
  return {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Headers':'Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token',
    'Access-Control-Request-headers': 'Content-Type, Authorization',
    'Access-Control-Request-method': 'GET',
    'Accept': '*/*',
    'Authorization': get_token()
  };
}

export async function ping() {
  const response = await fetch(as_url("/api/ping"), {
    method: "GET",
    headers: headers()
  });
  console.log(response)
  const res = await response.json();
  console.log(res)
  return res;
}
