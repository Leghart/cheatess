<script lang="ts">
  let socket: WebSocket;
  let commands: { action: string; parameters: string[] }[] = [];
  let action = "";
  let parameters = "";

  function connect() {
    socket = new WebSocket("ws://localhost:5555/ws/game");

    socket.onopen = () => {
      console.log("Connected to WebSocket server");
    };

    socket.onmessage = (event: MessageEvent) => {
      const command = JSON.parse(event.data);
      commands = [...commands, command];
    };

    socket.onclose = () => {
      console.log("WebSocket connection closed");
      setTimeout(connect, 1000);
    };

    socket.onerror = (error) => {
      console.error("WebSocket error", error);
    };
  }

  function sendCommand() {
    if (action && socket.readyState === WebSocket.OPEN) {
      const command = {
        action: action,
        parameters: parameters.split(",").map((p) => p.trim())
      };
      socket.send(JSON.stringify(command));
      action = "";
      parameters = "";
    }
  }

  connect();
</script>

<div class="command-interface">
  <div class="command-container">
    {#each commands as command}
      <p>
        <strong>Action:</strong>
        {command.action}, <strong>Parameters:</strong>
        {command.parameters.join(", ")}
      </p>
    {/each}
  </div>
  <input type="text" bind:value={action} placeholder="Action" />
  <input
    type="text"
    bind:value={parameters}
    placeholder="Parameters (comma separated)"
  />
  <button on:click={sendCommand}>Send Command</button>
</div>

<style>
  .command-container {
    border: 1px solid #ccc;
    padding: 1rem;
    height: 200px;
    overflow-y: auto;
    margin-bottom: 1rem;
    background-color: white;
    color: black;
  }
</style>
