<script>

  let text = ''

  import { endpoint } from './config.js'
  import { get_token, signin } from './auth'

  function as_url(path) {
    let url = endpoint + path;
    return url;
  }

  function headers() {
    return {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Headers':'Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token,Access-Control-Allow-Origin',
      'Access-Control-Request-headers': 'Content-Type, Authorization',
      'Accept': '*/*'
    };
  }

  async function list_notes() {
    const response = await fetch(as_url("/api/notes"), {
      method: "GET",
      headers: headers()
    });
    const res = await response.json();
    return res;
  }

  async function add_note(text) {
    const response = await fetch(as_url("/api/note"), {
      method: "POST",
      headers: headers(),
      body: JSON.stringify(
	{
	  'text': text
	}
      )
    });
    const res = await response.json();
    return res;
  }

  async function post_message() {
    await add_note(text)
  }


</script>

<div>
  <textarea bind:value={text}></textarea>
  <br/>
  <button on:click={post_message}>Post Message</button>
</div>
