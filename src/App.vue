<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Slider from "./components/Slider.vue";
import Tooltip from "./components/Tooltip.vue";

const appList = ref([]);
const apps: any = ref([]);

async function getApps() {
  appList.value = await invoke("get_apps", { apps: apps.value });
  console.log(apps.value);
}

onMounted(() => {
  getApps();
});
</script>

<template>
  <main class="container">
    <Tooltip />
    <Slider :items="appList" />
  </main>
</template>

<style>
:root {
  font-family: "Open Sans", Inter, Avenir, Helvetica, Arial, sans-serif;

  font-size: 16px;

  color: #0f0f0f;

  background-image: url("./assets/aa.jpg");

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}
</style>
