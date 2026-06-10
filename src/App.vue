<script setup lang="ts">
import Slider from "./components/Slider.vue";
import Tooltip from "./components/Tooltip.vue";
import AppEditor from "./components/AppEditor.vue";
import { ref } from "vue";

const tooltipText = ref("");
const tooltipVisible = ref(false);
const editorVisible = ref(false);

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
    <div class="dock-wrapper">
      <Slider @hover="showTooltip" @leave="hideTooltip" />
      <button class="settings-btn" @click="editorVisible = true" title="Configure Applications">
        ⚙️
      </button>
    </div>
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

.dock-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-btn {
  position: absolute;
  right: -24px;
  bottom: 16px;
  background: rgba(20, 20, 30, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(12px);
  color: #fff;
  border-radius: 50%;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  font-size: 1.1rem;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 10;
}

.settings-btn:hover {
  background: rgba(99, 102, 241, 0.8);
  border-color: rgba(99, 102, 241, 0.9);
  transform: rotate(45deg) scale(1.1);
  box-shadow: 0 4px 16px rgba(99, 102, 241, 0.4);
}

body {
  overflow: hidden; /* Hide scrollbars */
}
</style>
