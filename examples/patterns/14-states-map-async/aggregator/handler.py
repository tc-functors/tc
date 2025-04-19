
def handler(event, context):
    print(event)
    print("done")
    return {"status": event}
