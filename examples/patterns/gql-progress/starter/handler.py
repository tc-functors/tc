import json
import boto3
import time

def handler(input, context):
  print('received event:')
  print(input)
  event = input.get('arguments')
  id = event.get('id')

  time.sleep(1)

  data = {
    "id": id,
    "status": "startJob",
    "message": "Hello from starter lambda"
  }
  payload = {
    "data": data
  }


  print(payload)

  client = boto3.client('events')
  res = client.put_events(
    Entries=[
      {
        'Source': 'adHoc',
        'EventBusName': 'default',
        'Detail': json.dumps(payload),
        'DetailType': 'CompleteTask'
      }
    ]
  )
  print(res)
  return data
