<script setup lang="ts">
import Slider from "./components/Slider.vue";
// import Tooltip from "./components/Tooltip.vue";
// import { ref } from "vue";

// const hoveredApp = ref(null);

// function onHover(app: any) {
//   hoveredApp.value = app;
// }
</script>

<template>
  <main class="container hide-scrollbar">
    <!-- <Tooltip :app="hoveredApp" /> -->
    <Slider />
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
  margin-top: 3rem;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}
</style>
