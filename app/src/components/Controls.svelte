<script lang="ts">
  function handleStart() {
    console.log("Started scanning");
  }
  function handleStop() {
    console.log("Stopped scanning");
  }
</script>

<div class="controls">
  <button on:click={handleStart} class="btn start">Start</button>
  <button on:click={handleStop} class="btn stop">Stop</button>
</div>

<style>
  .controls {
    display: flex;
    flex-direction: column;
    color: white;
    gap: 1em;
  }
  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
    color: white;
  }

  .start {
    background-color: #28a745;
  }
  .stop {
    background-color: #dc3545;
  }
</style>
