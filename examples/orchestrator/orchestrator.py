import http.client
import json
import boto3

def make_mutation_str(mutation_name, input, output):
  inp = ''
  for k,v in input.items():
    inp += f'${k}:{v},'
  inp = inp.rstrip(',')
  fields = ''

  mut_input = ''
  for k,v in input.items():
    mut_input += f'{k}:${k},'
  mut_input = mut_input.rstrip(',')

  for k in output.keys():
    fields += f'{k} '
  fields += 'createdAt updatedAt'
  query = f'mutation({inp}){{{mutation_name}({mut_input}){{{fields}}}}}'
  return query


def make_mutation_payload(mutation_name, input, output, variables):
  mut_str = make_mutation_str(mutation_name, input, output)
  variables = json.dumps(variables)
  graphql_mutation = {
    'query': mut_str,
    'variables': f'{variables}'
  }
  return json.dumps(graphql_mutation)

def trigger_mutation(mutation_metadata, variables):
  mutation_name = mutation_metadata.get('name')
  input = mutation_metadata.get('input')
  output = mutation_metadata.get('output')
  endpoint = mutation_metadata.get('endpoint')
  api_key = mutation_metadata.get('api_key')
  host = endpoint.replace('https://','').replace('/graphql','')

  conn = http.client.HTTPSConnection(host, 443)
  headers = {
        'Content-type': 'application/graphql',
        'x-api-key': api_key,
        'host': host
    }
  rest_payload = make_mutation_payload(mutation_name, input, output, variables)
  conn.request('POST', '/graphql', rest_payload, headers)
  response = conn.getresponse()
  response_string = response.read().decode('utf-8')
  print(response_string)
  return response_string

def trigger_event(event_metadata, payload):
  client = boto3.client('events')
  res = client.put_events(
    Entries=[
      {
        'Source': event_metadata.get('source'),
        'EventBusName': event_metadata.get('bus'),
        'Detail': json.dumps(payload),
        'DetailType': event_metadata.get('name')
      }
    ]
  )
  print(res)
  return res

def trigger_function(function_arn, payload):
  client = boto3.client('lambda')
  response = client.invoke_async(
        FunctionName=function_arn,
        InvokeArgs=json.dumps(payload)
  )
  print(response)
  return response

def trigger_channel(channel_metadata, payload):
  host = channel_metadata.get('http_domain')
  api_key = channel_metadata.get('api_key')
  conn = http.client.HTTPSConnection(host, 443)
  headers = {
    'Content-type': 'application/json',
    'x-api-key': api_key,
  }
  body = {
    'channel': channel_metadata.get('name'),
    'events': [json.dumps(payload)]
  }
  conn.request('POST', '/event', json.dumps(body), headers)
  response = conn.getresponse()
  response_string = response.read().decode('utf-8')
  print(response_string)
  return response_string

def trigger_targets(targets, payload):
  event_metadata = targets.get("event")
  mutation_metadata = targets.get("mutation")
  function_arn = targets.get("function")
  channel_metadata = targets.get("channel")
  if event_metadata is not None:
    trigger_event(event_metadata, payload)

  if function_arn is not None:
    trigger_function(function_arn, payload)

  if mutation_metadata is not None:
    trigger_mutation(mutation_metadata, payload)

  if channel_metadata is not None:
    trigger_channel(channel_metadata, payload)
  return True

def load_metadata(source_arn):
  with open('orchestrator.json') as json_data:
    d = json.load(json_data)
    targets = d.get('targets').get(source_arn)
    json_data.close()
    return targets

def make_input(response_payload):
  if 'detail' in response_payload and 'detail-type' in response_payload:
    return response_payload.get('detail')
  else:
    return response_payload


def handler(event, context):
  print(event)
  input = make_input(event.get('responsePayload'))
  s = event.get('requestContext').get('functionArn')
  source_arn = s.rsplit(':', 1)[0]
  targets = load_metadata(source_arn)
  res = trigger_targets(targets, input)
  return res
