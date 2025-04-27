
import numpy

def handler(event, context):
  print(numpy.version.version)
  print('hello world')
  return event
