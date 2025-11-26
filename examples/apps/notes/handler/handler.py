import uuid

def handler(event, context):
  id = uuid.uuid4()
  return {'id': 1, 'text': event.get('text')}
