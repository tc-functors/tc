import App from './App.svelte';
import { Amplify } from 'aws-amplify';
import config from './config.json';

Amplify.configure(config);

const app = new App({
  target: document.body
});

export default app;
