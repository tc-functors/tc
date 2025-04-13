import json
import boto3
import time

def handler(input, context):
  print('received event:')
  print(input)
  event = input.get('arguments')
  id = event.get('id')

  time.sleep(1)

  payload = {
    "id": id,
    "status": "startJob",
    "message": "Hello from starter lambda"
  }

  print(payload)

  client = boto3.client('events')
  res = client.put_events(
    Entries=[
      {
        'Source': 'adHoc',
        'EventBusName': 'techno-core-dev-af',
        'Detail': json.dumps(payload),
        'DetailType': 'CompleteTask'
      }
    ]
  )
  print(res)
  return payload
