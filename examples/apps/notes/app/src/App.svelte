<script lang="ts">
  import { onMount } from 'svelte';
  import { get_user, signin, is_signed_in,
	   signin_callback, signout, get_user_name } from './auth.js'

  import Publisher from './Publisher.svelte'
  import Subscriber from './Subscriber.svelte'

  let is_authenticated = true
  let user, user_name

  onMount(() => {

    is_authenticated = is_signed_in()

    if (!is_authenticated) {
      signin_callback()
    } else {
      user = get_user()
      user_name = get_user_name()
    }

  });

 function do_signin() {
   signin()
   window.location.reload();
 }


</script>


<main class="container">


<div class="grid">

  <Publisher/>

  <Subscriber/>

</div>

</main>

<style>

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    width: 100%;
    max-width: 80rem;
    margin: 0 auto;
    box-sizing: border-box;
  }
</style>
