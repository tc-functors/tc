export const endpoint = "{{REST_ENDPOINT}}"

export const oidc_config = {
  authority: "{{OIDC_AUTHORITY}}",
  client_id: "{{OIDC_CLIENT_ID}}",
  redirect_uri: "http://localhost:5173",
  cognito_domain: "http://localhsot:5173",
  response_type: "code",
  scope: "email openid",
}

export const graphql_config = {
  aws_project_region: "us-west-2",
  aws_appsync_graphqlEndpoint: "{{GRAPHQL_ENDPOINT}}",
  aws_appsync_region: "us-west-2",
  aws_appsync_authenticationType: "AWS_LAMBDA"
}
