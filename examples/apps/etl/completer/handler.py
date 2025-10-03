import json
import time

def parse_detail(detail):
  detail = detail.strip('{').strip('}')
  return dict(i.split("=") for i in detail.split(", "))

def handler(event, context):
  print('received event:')
  print(event)
  args = event.get('arguments')
  detail = parse_detail(args.get('detail'))
  id = detail.get('id')

  time.sleep(2)

  response = {
    'id': id,
    'status': "Completed",
    'message': "",
    'percentage': 100
  }

  print(response)
  return response
