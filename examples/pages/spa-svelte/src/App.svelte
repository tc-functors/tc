<script>
  import { Router, Link, Route } from "svelte-routing";

  import Home from './Home.svelte'
  import Foo from './Foo.svelte'
  import Bar from './Bar.svelte'
  import Baz from './Baz.svelte'
</script>

<main>

  <Router>
    <nav>
      <Link to="/">Home</Link>
      <Link to="/foo">Foo</Link>
      <Link to="/bar">Bar</Link>
      <Link to="/baz">Baz</Link>
    </nav>

    <div class="box">
      <Route path="/" component={Home}/>
      <Route path="/foo" component={Foo}/>
      <Route path="/bar" component={Bar}/>
      <Route path="/baz" component={Baz}/>

    </div>
  </Router>

</main>


<style>

  main {
    text-align: center;
    padding: 1em;
    max-width: 240px;
    margin: 0 auto;
  }

  .box {
    padding-top: 25px;
  }

  @media (min-width: 640px) {
    main {
      max-width: none;
    }
  }
</style>
