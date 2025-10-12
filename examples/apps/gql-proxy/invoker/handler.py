import http.client
import json
import boto3

API_URL = ''
API_KEY = ''

HOST = API_URL.replace('https://','').replace('/graphql','')

types = {
  "processMessage": {
    'input': {
      'id': 'String!',
      'text': 'String'
    },
    'output': {
      'id': 'String!',
      'text': 'String'
    }
  }
}

def make_mutation_str(mutation_name):
  m = types.get(mutation_name)
  input = ''
  for k,v in m.get('input').items():
    input += f'${k}:{v},'
  input = input.rstrip(',')
  fields = ''

  mut_input = ''
  for k,v in m.get('input').items():
    mut_input += f'{k}:${k},'
  mut_input = mut_input.rstrip(',')

  for k in m.get('output').keys():
    fields += f'{k} '
  fields += 'createdAt updatedAt'
  str = f'mutation({input}){{{mutation_name}({mut_input}){{{fields}}}}}'
  return str

def make_mutation_payload():
    graphql_mutation = {
        'query': 'mutation($id:String!,$text:String){processMessage(id:$id,text:$text){id text createdAt updatedAt}}',
        'variables': '{"id":"abc", "text":"hola"}'
    }
    return json.dumps(graphql_mutation)

def trigger_mutation():
  conn = http.client.HTTPSConnection(HOST, 443)
  headers = {
        'Content-type': 'application/graphql',
        'x-api-key': API_KEY,
        'host': HOST
    }
  payload = make_mutation_payload()
  conn.request('POST', '/graphql', payload, headers)
  response = conn.getresponse()
  response_string = response.read().decode('utf-8')
  return response_string

def handler(event, context):
  res = trigger_mutation()
  return res
