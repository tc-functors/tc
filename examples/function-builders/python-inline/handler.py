import yaml

def handler(event, context):
    print(yaml.dump([1,2,3], explicit_start=True))
    return event
