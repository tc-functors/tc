from aws_lambda_powertools import Logger
from aws_lambda_powertools.utilities import parameters

logger = Logger()

def handler(event, context):
    try:
      # Get secret with caching (default TTL: 5 seconds)
      secret_value = parameters.get_secret("my-secret-name")

      # Get secret with custom TTL
      secret_with_ttl = parameters.get_secret("my-secret-name", max_age=300)

      # Get secret and transform JSON
      secret_json = parameters.get_secret("my-json-secret", transform="json")

      logger.info("Successfully retrieved secrets")

      return {
        'statusCode': 200,
        'body': 'Successfully retrieved secrets'
      }

    except Exception as e:
      logger.error(f"Error retrieving secret: {str(e)}")
      return {
        'statusCode': 500,
        'body': f'Error: {str(e)}'
      }
