
def handler(event, context):
  print("Triggered bar from foo")
  print(event)
  return event
