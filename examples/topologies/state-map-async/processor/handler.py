import boto3
import json

def handler(event, context):
    # do some really computationally expensive thing

    token = event.get('token')
    client = boto3.client("stepfunctions")
    response = {'data': "processed data"}
    client.send_task_success(taskToken=token, output=json.dumps(response))


    return {
        "index": event['index'],
        "detail": event['detail'],
        "token": event['token']
    }
