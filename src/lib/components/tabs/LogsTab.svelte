<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { Download, Trash2, Pause, Play } from 'lucide-svelte';

  interface LogsTabData {
    contextName: string;
    namespace: string;
    podName: string;
    containerName: string;
    streamId: string;
  }

  let { data }: { data: LogsTabData } = $props();

  let logs = $state<string[]>([]);
  let unlisten: (() => void) | null = null;
  let isPaused = $state(false);
  let autoScroll = $state(true);
  let containerRef: HTMLDivElement;

  async function startStreaming() {
    try {
      await invoke('stream_container_logs', {
        contextName: data.contextName,
        namespace: data.namespace,
        podName: data.podName,
        containerName: data.containerName,
        streamId: data.streamId,
      });
    } catch (e) {
      console.error('Failed to start log stream:', e);
      logs.push(`[Error] Failed to start log stream: ${e}`);
    }
  }

  onMount(async () => {
    const eventName = `container_logs_${data.streamId}`;
    unlisten = await listen<string>(eventName, (event) => {
      if (!isPaused) {
        logs.push(event.payload);
        if (autoScroll && containerRef) {
          // Schedule scroll for next tick
          setTimeout(() => {
            if (containerRef) {
              containerRef.scrollTop = containerRef.scrollHeight;
            }
          }, 0);
        }
      }
    });

    startStreaming();
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });

  function clearLogs() {
    logs = [];
  }

  function downloadLogs() {
    const blob = new Blob([logs.join('\n')], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${data.containerName}.log`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function togglePause() {
    isPaused = !isPaused;
  }
</script>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-panel border-b border-border">
    <div class="flex items-center gap-2">
      <span class="text-sm text-text-muted">
        {data.podName} / {data.containerName}
      </span>
      <span class="text-xs text-text-muted">
        ({logs.length} lines)
      </span>
    </div>
    <div class="flex items-center gap-2">
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={autoScroll} class="rounded" />
        Auto-scroll
      </label>
      <button
        class="p-1.5 hover:bg-bg-main rounded transition-colors"
        onclick={togglePause}
        title={isPaused ? 'Resume' : 'Pause'}
      >
        {#if isPaused}
          <Play size={16} />
        {:else}
          <Pause size={16} />
        {/if}
      </button>
      <button
        class="p-1.5 hover:bg-bg-main rounded transition-colors"
        onclick={clearLogs}
        title="Clear logs"
      >
        <Trash2 size={16} />
      </button>
      <button
        class="p-1.5 hover:bg-bg-main rounded transition-colors"
        onclick={downloadLogs}
        title="Download logs"
      >
        <Download size={16} />
      </button>
    </div>
  </div>

  <!-- Logs Content -->
  <div 
    bind:this={containerRef}
    class="flex-1 overflow-auto p-4 font-mono text-xs bg-bg-main"
  >
    {#each logs as line, i}
      <div class="hover:bg-bg-panel/50">
        <span class="text-text-muted select-none mr-4">{i + 1}</span>
        <span class="text-text">{line}</span>
      </div>
    {/each}
  </div>
</div>
