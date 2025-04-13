import json
import boto3
import time
import os

def parse_detail(detail):
  detail = detail.strip('{').strip('}')
  return dict(i.split("=") for i in detail.split(", "))

def handler(input, context):
  print('received event:')
  args = input.get('arguments')
  detail = parse_detail(args.get('detail'))
  id = detail.get('id')

  time.sleep(1)

  payload = {
    "id": id,
    "status": "Applying LLM and transforming data",
    "message": "Transformed",
    "percentage": 75
  }

  print(payload)
  env = os.environ.get('Environment', 'dev-af')

  client = boto3.client('events')
  res = client.put_events(
    Entries=[
      {
        'Source': 'adHoc',
        'EventBusName': f'techno-core-{env}',
        'Detail': json.dumps(payload),
        'DetailType': 'TransformerComplete'
      }
    ]
  )
  print(res)
  return payload
