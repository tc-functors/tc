import boto3
import json

def invoke_sfn(payload):
    client = boto3.client('stepfunctions')
    response = client.start_sync_execution(
        stateMachineArn='{sfn_arn}',
        input=json.dumps(payload)
     )
    return response

def handler(event, context):
    print(event)
    response = invoke_sfn(event)
    output = response.get('output')
    return {{
        'statusCode': 200,
        'headers': {{"Content-Type": "application/json"}},
        'body': output
    }}
