<script>
  import { onMount } from 'svelte';
  import { generateClient } from 'aws-amplify/api';
  import { Amplify } from 'aws-amplify';
  import config from './config.json';

  let subs = [];
  let error = '';

  let auth_token = 'auth123';
  let client;

  const addNote = /* GraphQL */ `
  mutation AddNote($id: String!) {
    addNote(id: $id) {
      id
      text
      createdAt
      updatedAt
      __typename
    }
  }
`;

  const subscribeAddNote = /* GraphQL */ `
  subscription SubscribeAddNote($id: String!) {
    subscribeStartJob(id: $id) {
      id
      text
      createdAt
      updatedAt
      __typename
    }
  }
`;

  onMount(async () => {
    let a = Amplify.configure(config);
    console.log("conf", a);
    client = generateClient();
  })

  async function start() {
    console.log("subscribing addNote: ");
    const sub_response1 = await client.graphql({
      query: subscribeAddNote,
      variables: {
	id: 1
      },
      authToken: auth_token
    }).subscribe({
      next: ({ data }) => {
	console.log("got data: ", data)
	console.log(data.subscribeAddNote)
	subs.push(data.subscribeAddNote);
	subs = subs;
      },
      error: (error) => console.error(error)
    });

  }

</script>


<main>
  <div>
    {#if subs }
      {#each subs as sub}
  <br />
  <blockquote>
    {sub.text}
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
