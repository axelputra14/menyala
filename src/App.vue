<script setup lang="ts">
import Slider from "./components/Slider.vue";
import Tooltip from "./components/Tooltip.vue";
import AppEditor from "./components/AppEditor.vue";
import { ref, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const tooltipText = ref("");
const tooltipVisible = ref(false);
const editorVisible = ref(false);

let unlistenShowEditor: UnlistenFn | undefined;

onMounted(async () => {
  unlistenShowEditor = await listen("show-editor", () => {
    editorVisible.value = true;
  });
});

onUnmounted(() => {
  if (unlistenShowEditor) {
    unlistenShowEditor();
  }
});

function showTooltip(text: string) {
  tooltipText.value = text;
  tooltipVisible.value = true;
}

function hideTooltip() {
  tooltipVisible.value = false;
}
</script>

<template>
  <main class="container hide-scrollbar">
    <div class="flexbox">
      <Tooltip :text="tooltipText" :visible="tooltipVisible" />
    </div>

    <Slider @hover="showTooltip" @leave="hideTooltip" />

    <AppEditor :visible="editorVisible" @close="editorVisible = false" />
  </main>
</template>

<style>
:root {
  font-family: "Open Sans", Inter, Avenir, Helvetica, Arial, sans-serif;

  font-size: 16px;

  color: #0f0f0f;

  background-color: transparent;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin-top: 0.25rem;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  text-align: center;
  position: relative;
}

.flexbox {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

body {
  overflow: hidden; /* Hide scrollbars */
}
</style>
