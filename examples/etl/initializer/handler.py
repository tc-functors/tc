import json
import boto3
import time
import os

def handler(input, context):
  print('received event:')
  print(input)
  event = input.get('arguments')
  id = event.get('id')

  time.sleep(1)

  payload = {
    "id": id,
    "status": "Initalized ETL",
    "message": "Initialized",
    "percentage": 25
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
        'DetailType': 'InitializationComplete'
      }
    ]
  )
  print(res)
  return payload
