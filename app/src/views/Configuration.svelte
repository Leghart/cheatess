<script lang="ts">
  import Board from "../components/Board.svelte";
  import { Steps } from "svelte-steps";

  let steps = [
    { text: "Region" },
    { text: "Platform" },
    { text: "Theme" },
    { text: "Test" }
  ];

  let selectedPlatform: "Chesscom" | "Lichess" | null = null;
  let themeChoice: "Default" | "Custom" | null = null;
  let showModal = false;
  let thresholdValues = Array(12).fill(0.0);
  let activeStep = 0;

  const BACKEND_URL = "http://127.0.0.1:5555/config";

  function handlePlatformChange(platform: "Chesscom" | "Lichess") {
    selectedPlatform = selectedPlatform === platform ? null : platform;
  }

  function openModal() {
    showModal = true;
  }

  function closeModal() {
    showModal = false;
  }

  async function sendReq() {
    let body = {
      platform: selectedPlatform,
      theme: themeChoice,
      thresholds: thresholdValues
    };
    const response = await fetch(BACKEND_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
  }

  function isStepValid() {
    if (activeStep === 0) {
      return selectedPlatform !== null;
    }
    if (activeStep === 1) {
      return themeChoice !== null;
    }
    if (activeStep === 2) {
      return thresholdValues.every((value) => !isNaN(value));
    }
    return false;
  }

  function goToNextStep() {
    if (isStepValid() && activeStep < steps.length - 1) {
      activeStep += 1;
    }
  }

  function goToPreviousStep() {
    if (activeStep > 0) {
      activeStep -= 1;
    }
  }
</script>

<main class="app-container {activeStep === 3 ? 'board-mode' : 'full-width'}">
  <div class="controls-section">
    <div class="steps-container">
      <Steps {steps} bind:current={activeStep} />
    </div>

    {#if activeStep === 0}{/if}

    <!-- Platform Step -->
    {#if activeStep === 1}
      <div class="step-content">
        <label>
          <input
            type="checkbox"
            checked={selectedPlatform === "Chesscom"}
            on:change={() => handlePlatformChange("Chesscom")}
          />
          Chesscom
        </label>
        <label>
          <input
            type="checkbox"
            checked={selectedPlatform === "Lichess"}
            on:change={() => handlePlatformChange("Lichess")}
          />
          Lichess
        </label>
      </div>
    {/if}

    <!-- Theme Step -->
    {#if activeStep === 2}
      <div class="step-content">
        <label>
          <input type="radio" bind:group={themeChoice} value="Default" /> Default
        </label>
        <label>
          <input type="radio" bind:group={themeChoice} value="Custom" /> Custom
        </label>
        {#if themeChoice === "Default"}
          <div>pieces: "default pieces", board: "default board"</div>
        {/if}
        {#if themeChoice === "Custom"}
          <button on:click={openModal}>Open Modal</button>
          {#if showModal}
            <div class="modal">
              <div class="modal-content">
                <p>abc</p>
                <button on:click={closeModal}>Close</button>
              </div>
            </div>
          {/if}
        {/if}
      </div>
    {/if}

    <!-- Test Step -->
    {#if activeStep === 3}
      <div class="step-content threshold-column">
        {#each thresholdValues as value, index}
          <label>
            Threshold {index + 1}:
            <input
              type="number"
              step="0.01"
              bind:value={thresholdValues[index]}
            />
          </label>
        {/each}
      </div>
    {/if}

    <div class="navigation-buttons">
      <button on:click={goToPreviousStep} disabled={activeStep === 0}
        >Previous</button
      >
      <button on:click={goToNextStep} disabled={!isStepValid()}>Next</button>
    </div>

    {#if activeStep === 3}
      <div>
        <button on:click={sendReq}> Send</button>
      </div>
    {/if}
  </div>

  <!-- Board Section only on Test Step -->
  {#if activeStep === 4}
    <div class="board-section">
      <Board />
    </div>
  {/if}
</main>

<style>
  .app-container {
    display: grid;
    color: white;
    height: calc(100vh - 60px);
    overflow-y: hidden;
    background-color: #333;
  }

  .full-width {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .board-mode {
    display: grid;
    grid-template-columns: 1000px 1fr;
  }

  .controls-section {
    padding: 1rem;
    background-color: #2e2e2e;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .steps-container {
    display: flex;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .board-section {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .modal-content {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    color: black;
  }

  .navigation-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .threshold-column {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
</style>
