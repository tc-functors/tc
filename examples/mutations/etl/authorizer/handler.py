import json

def handler(event, context):
    print(event)
    token = event.get('authorizationToken')
    return {
        'isAuthorized': token == "auth123",
        'deniedFields': [],
        'ttlOverride': 3600
    }
