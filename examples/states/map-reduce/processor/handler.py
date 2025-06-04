
def handler(event, context):
    nums = event.get('data')
    return max(nums)
