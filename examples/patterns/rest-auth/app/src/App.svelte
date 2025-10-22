<script lang="ts">

  import { onMount } from 'svelte'
  import { get_user, signin, is_signed_in,
	   signin_callback, signout, get_user_name } from './auth'
  import { oidc_config } from './config'
  import { ping } from './api'

  let path = window.location.pathname;

  const updatePath = () => {
    path = window.location.pathname;
  };

  let is_authenticated = false
  let user = {};
  let user_name =  '';

  onMount(() => {
    is_authenticated = is_signed_in()

    if (!is_authenticated) {
      signin_callback()
    } else {
      user = get_user()
      user_name = get_user_name()
    }
  })
  let response = ''

  async function api_ping() {
    const res = await ping();
    response = res.status
  }

</script>

<div class="container">

<br/>
<article>

<header>
Config:
</header>

Authority: {oidc_config.authority} <br/>
client_id: {oidc_config.client_id} <br/>
redirect_uri: {oidc_config.redirect_uri} <br/>
response_type: {oidc_config.response_type} <br/>
scope: {oidc_config.scope} <br/>
</article>

{#if  is_authenticated }

  <p>Logged in as {user_name}</p>

  <br/>

  <article>

  <header>
    Authenticated API
  </header>

  <br/>
  <button on:click={api_ping}>Ping</button>
  <br/>
  Response: {response}
  </article>

  <br/>

  <button on:click={signout}>Signout</button>

{:else}

<div>
  <div align="center">
    <br/>
    <br/>
    <button on:click={signin}>Sign In</button>
  </div>
</div>

{/if}

</div>
