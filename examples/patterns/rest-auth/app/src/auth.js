import { UserManager } from "oidc-client-ts"
import { oidc_config } from './config'

export function get_user() {
  return window.localStorage.getItem('user');
}

export function get_user_name() {
  const user = get_user();
  if (user) {
    const p = JSON.parse(user)
    return p.profile.email
  } else {
    return ""
  }
}

export function get_token() {
  const user = get_user();
  if (user) {
    const p = JSON.parse(user)
    return p.id_token
  } else {
    return ""
  }
}

function set_user(user) {
  window.localStorage.setItem('user', JSON.stringify(user));
}

export function is_signed_in() {
  const user = get_user();
  if (user) {
    return true
  } else {
    return false
  }
}

export const userManager = new UserManager(oidc_config);

export function signin() {
  userManager.signinRedirect();
}

export function signin_callback() {
  userManager.signinRedirectCallback().then(function(user) {
    console.log("User logged in", user);
    return true
  }).catch(function(err) {
    console.error(err);
    return false
  });

  userManager.getUser().then(function(user) {
    if (user) {
      set_user(user)
      console.log("User info:", user.profile);
    } else {
      console.log("User not logged in");
    }
  });
}

export function maybe_signin() {
  const user = get_user()
  if (user) {
    return true
  } else {
    console.log("No user")
    //return false
    return signin()
  }
}

export async function signout() {
  // const clientId = config.client_id;
  // const logoutUri = config.redirect_uri;
  // const cognitoDomain = "https://example-auth.auth.us-west-2.amazoncognito.com";
  // window.localStorage.removeItem('user');
  // window.location.href = `${cognitoDomain}/logout?client_id=${clientId}&logout_uri=${encodeURIComponent(logoutUri)}`;
  //window.location.reload();
}
