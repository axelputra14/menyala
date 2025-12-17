<script setup lang="ts">
import Slider from "./components/Slider.vue";
import Tooltip from "./components/Tooltip.vue";
import { ref } from "vue";

const tooltipText = ref("");
const tooltipVisible = ref(false);

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
  margin-top: 1rem;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
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
