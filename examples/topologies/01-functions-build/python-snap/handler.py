from importer import LambdaImporter

def handler(event, context):
  imp = LambdaImporter(bucket_name=BUCKET)
  a = imp.import_package("torch_zip", "libtorch.zip")
  print(a)
  b = imp.import_package("torch", "torch.whl")
  print(b)
  return {'data': 123}
