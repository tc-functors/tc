
import numpy

def handler(event, context):
  print(numpy.version.version)
  return event
