<script lang="ts">
  import Board from "../components/Board.svelte";
  import Controls from "../components/Controls.svelte";
  import Logs from "../components/Logs.svelte";
  import MovesInfo from "../components/MovesInfo.svelte";
  import Socket from "../Socket.svelte";
</script>

<main class="app-container">
  <div class="controls-section">
    <Controls />
    <Logs />
  </div>
  <div class="board-section">
    <div class="board-container">
      <Board />
      <MovesInfo />
    </div>
  </div>
  <div>
    <Socket />
  </div>
</main>

<style>
  .app-container {
    display: grid;
    grid-template-columns: 250px 1fr;
    height: calc(100vh - 60px);
    background-color: #1e1e1e;
    color: white;
  }

  .controls-section {
    padding: 1rem;
    background-color: #2e2e2e;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .board-section {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .board-container {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    height: 100%;
  }
</style>
