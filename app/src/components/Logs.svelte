
<!-- Components/Logs.svelte -->
<script lang="ts">
    let logs: string[] = ['Started scanning board'];
    function handleClear() {
      console.log("Logs cleared");
    }    
</script>


<h3>Logs</h3>
<div class="logs">
    <ul>
        {#each logs as log}
            <li>{log}</li>
        {/each}
    </ul>
</div>
<button on:click="{handleClear}" class="btn clear">Clear logs</button>




<style>
    .logs {
        background-color: #444;
        border-radius: 8px;
        color: #00ff00;
        overflow-y: auto;
        flex-grow: 1;
    }
    .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
    color: white;
  }
    
    .clear {
    background-color: #6c757d;
  }    
</style>