
INIT = False

if not INIT:
  print("Initializing")

def handler(event, context):
  INIT = True
  return {'data': 123}
