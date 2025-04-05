from import_from_s3 import S3WheelImporter

def handler(event, context):
  bucket = 'informed-techno-core-dev-af-logs'
  importer = S3WheelImporter(bucket, prefix='wheels/')

  numpy = importer.import_package('numpy')
  #torch = importer.import_package('torch')

  if numpy:
    print(f"Successfully imported numpy version {numpy.__version__}")
    print(numpy.array([1, 2, 3]))

  # if torch:
  #   print(f"Successfully imported torch version {torch.__version__}")
  #   print(torch.tensor([1, 2, 3]))

  return {'data': 123}
