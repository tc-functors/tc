export const endpoint = "{{REST_ENDPOINT}}"

export const oidc_config = {
  authority: "{{OIDC_AUTHORITY}}",
  client_id: "{{OIDC_CLIENT_ID}}",
  cognito_domain: "",
  redirect_uri: "http://localhost:5173",
  response_type: "code",
  scope: "phone openid email"
}
