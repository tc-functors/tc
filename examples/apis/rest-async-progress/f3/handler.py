import json
import boto3

def handler(event, context):
  print('received event:')
  client = boto3.client('events')
  payload = {"message": "from f3. Completed processing"}
  res = client.put_events(
    Entries=[
      {
        'Source': 'default',
        'EventBusName': 'default',
        'Detail': json.dumps(payload),
        'DetailType': 'ProcessComplete'
      }
    ]
  )
  print(res)
  return {'message': 'from f3'}
