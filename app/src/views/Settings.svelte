<script lang="ts">
  let stockfishDepth = 17;
  let memoryUsage = 1024;
  let eloLevel = 2200;
  let stockfishPath = "/path/to/stockfish";

  const BACKEND_URL = "http://127.0.0.1:5555/stockfish/config";

  function resetSettings() {
    stockfishDepth = 17;
    memoryUsage = 1024;
    eloLevel = 2200;
    stockfishPath = "/path/to/stockfish";
  }

  async function saveSettings() {
    let body = {
      depth: stockfishDepth,
      memory: memoryUsage,
      elo: eloLevel,
      engine_path: stockfishPath
    };
    const response = await fetch(BACKEND_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
  }

  async function getSettings() {
    const response = await fetch(BACKEND_URL, {
      method: "GET",
      headers: { "Content-Type": "application/json" }
    });
    console.log(await response.json());
  }
</script>

<div class="settings-page">
  <div class="settings-content">
    <div class="input-group">
      <label for="config-depth">Stockfish depth (1-20):</label>
      <input
        id="config-depth"
        type="number"
        bind:value={stockfishDepth}
        min="1"
        max="20"
      />

      <label for="config-memory">Amount of memory to use [MB]:</label>
      <input id="config-memory" type="number" bind:value={memoryUsage} />
    </div>

    <p>ELO level: {eloLevel}</p>
    <input
      type="range"
      min="1000"
      max="3200"
      step="100"
      bind:value={eloLevel}
    />

    <label for="config-elo">Path to downloaded stockfish engine:</label>
    <input
      id="config-elo"
      type="text"
      bind:value={stockfishPath}
      class="path-input"
    />

    <div class="button-group">
      <button class="btn reset" on:click={resetSettings}>Reset settings</button>
      <button class="btn save" on:click={saveSettings}>Save</button>
      <button class="btn save" on:click={getSettings}>Load</button>
    </div>
  </div>
</div>

<style>
  .settings-page {
    padding: 2rem;
    color: white;
    height: calc(100vh - 60px);
    overflow-y: hidden;
    background-color: #333;
  }

  .input-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .settings-content {
    background-color: #2e2e2e;
    padding: 1rem;
    border-radius: 8px;
    height: 100%;
    overflow-y: hidden;
  }

  .path-input {
    width: 100%;
    padding: 0.5rem;
    border-radius: 4px;
    border: none;
    margin-top: 0.5rem;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .btn {
    padding: 0.5rem 1.5rem;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
    color: white;
  }
  .reset {
    background-color: #007bff;
  }
  .save {
    background-color: #007bff;
  }

  ::-webkit-scrollbar {
    width: 0;
    height: 0;
  }
</style>
