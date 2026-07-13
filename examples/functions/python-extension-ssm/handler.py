import json
import os
import requests

def lambda_handler(event, context):
    try:
      # Replace with the name or ARN of your secret
      secret_name = "arn:aws:secretsmanager:us-east-1:111122223333:secret:SECRET_NAME"

      secrets_extension_endpoint = f"http://localhost:2773/secretsmanager/get?secretId={secret_name}"
      headers = {"X-Aws-Parameters-Secrets-Token": os.environ.get('AWS_SESSION_TOKEN')}

      response = requests.get(secrets_extension_endpoint, headers=headers)
      print(f"Response status code: {response.status_code}")

      secret = json.loads(response.text)["SecretString"]
      print(f"Retrieved secret: {secret}")

      return {
        'statusCode': response.status_code,
        'body': json.dumps({
          'message': 'Successfully retrieved secret',
          'secretRetrieved': True
        })
      }

    except Exception as e:
      print(f"Error: {str(e)}")
      return {
        'statusCode': 500,
        'body': json.dumps({
          'message': 'Error retrieving secret',
          'error': str(e)
        })
      }
