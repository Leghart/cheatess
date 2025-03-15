<script lang="ts">
  import Game from '../views/Game.svelte';
  import Settings from '../views/Settings.svelte';
  import Configuration from '../views/Configuration.svelte';
  import { writable } from 'svelte/store';

  const currentPage = writable('game');

  function setPage(page: string) {
      currentPage.set(page);
  }
</script>

<nav class="navbar">
  <div class="navbar-buttons">
      <button on:click={() => setPage('game')}>Game</button>
      <button on:click={() => setPage('settings')}>Settings</button>
      <button on:click={() => setPage('configuration')}>Configuration</button>
  </div>
</nav>

{#if $currentPage === 'game'}
  <Game/>
{/if}

{#if $currentPage === 'settings'}
  <Settings />
{/if}

{#if $currentPage === 'configuration'}
  <Configuration />
{/if}

<style>
  .navbar {
      background-color: #333;
      padding: 1rem;
      display: flex;
      justify-content: center;
  }
  .navbar-buttons {
      display: flex;
      gap: 1rem;
  }
  .navbar button {
      background-color: #555;
      color: white;
      border: none;
      padding: 0.5rem 1rem;
      border-radius: 8px;
      cursor: pointer;
  }
  .navbar button:hover {
      background-color: #777;
  }

</style>
