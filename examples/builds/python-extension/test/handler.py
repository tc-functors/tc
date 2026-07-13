import logging

logger = logging.getLogger()

def handler(event, context):
    logger.info("got an event")
    logger.info(event)
    print({'key': "test", 'value': event})
    return {'data': {}}
