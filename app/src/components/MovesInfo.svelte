<script lang="ts">
  let bestMove = "♘ d4";
  let topMoves = [
    { score: 0.3, move: "♘ d4" },
    { score: 0.1, move: "♕ d4" },
    { score: -0.6, move: "♘ b1" }
  ];
  let winDrawLossStats = "4-95-0";
</script>

<div class="moves-info">
  <p>Best move: {bestMove}</p>
  <p>Top moves:</p>
  <ul>
    {#each topMoves as { score, move }}
      <li>{score}: {move}</li>
    {/each}
  </ul>
  <p>Win-Draw-Loss stats (%): {winDrawLossStats}</p>
</div>

<style>
  .moves-info {
    background-color: #555;
    border-radius: 8px;
    padding: 1rem;
  }
</style>
