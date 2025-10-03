import json
import time

# TODO: for some reason detail is not JSON. Yet to figure out the transformation. This is a hack until then
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
    'status': 'completed',
    'message': "Bye from complete lambda"
  }

  print(response)
  return response
