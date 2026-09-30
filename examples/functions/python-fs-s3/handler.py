import os

def handler(event, context):
  path = "/mnt/assets"
  entries = os.listdir(path)
  return entries
