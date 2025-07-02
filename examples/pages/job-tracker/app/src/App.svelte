<script>
  import { onMount } from 'svelte';
  import { generateClient } from 'aws-amplify/api';
  import { startJob, subscribeStartJob, subscribeCompleteJob } from './graphql'
  import { Amplify } from 'aws-amplify';
  import config from './config.json';

  let job_id;
  let t_job_id;
  let subs = [];
  let error = '';

  let auth_token = '';
  let client;

  onMount(async () => {
    let a = Amplify.configure(config);
    console.log("conf", a);
    client = generateClient();
  })

  async function start() {
    console.log("subscribing startJob: ", job_id);
    const sub_response1 = await client.graphql({
      query: subscribeStartJob,
      variables: {
	id: job_id
      },
      authToken: auth_token
    }).subscribe({
      next: ({ data }) => {
	console.log("got data: ", data)
	console.log(data.subscribeStartJob)
	subs.push(data.subscribeStartJob);
	subs = subs;
      },
      error: (error) => console.error(error)
    });

    console.log("subscribing completeJob: ", job_id);
    const sub_response2 = await client.graphql({
      query: subscribeCompleteJob,
      variables: {
	id: job_id
      },
      authToken: auth_token
    }).subscribe({
      next: ({ data }) => {
	console.log("got data: ", data)
	console.log(data.subscribeCompleteJob)
	subs.push(data.subscribeCompleteJob);
	subs = subs;
      },
      error: (error) => console.error(error)
    });

    console.log("starting job: ", job_id)
    const start_response = await client.graphql({
      query: startJob,
      variables: {
	id: job_id
      },
      authToken: auth_token
    });
    console.log(start_response);

  }

</script>


<main>
  <div>
    <h1>Job Tracker</h1>

    <form on:submit|preventDefault={start}>

      <label  type="text" for="s1-2"> Job ID </label>
      <input type="text" bind:value={job_id} id="job_id" />
      <label  type="text" for="s1-2"> Token </label>
      <input type="text" bind:value={auth_token} id="auth_token" />
      <button type="submit">Start Job</button>

      {#if error}
	{error}
      {/if}
    </form>


    <br/>

    {#if subs }
      {#each subs as sub}

  <b> {sub.id} - {sub.status} </b>
  <br />
  <blockquote>
    {sub.message}
  </blockquote>

  <br/>
{/each}
 {/if}

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
