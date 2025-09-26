<script setup lang="ts">
import { defineProps } from "vue";

const props = defineProps({
  app: {
    type: Object as () => { id: number; name: string; icon: string } | null,
    default: null,
  },
});
</script>

<template>
  <div class="tooltip-container" v-if="app">
    <div class="text-box">
      <p>{{ app.name }}</p>
    </div>
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=Open+Sans:ital,wght@0,300..800;1,300..800&display=swap");

.tooltip-container {
  font-family: "Open Sans", sans-serif;
  font-weight: 500;
  font-size: 16px;
  line-height: 0.125rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 6px;
}

.text-box {
  background: rgba(0, 0, 0, 0.5);
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.17);
  /* backdrop-filter: blur(1px); */
  /* -webkit-backdrop-filter: blur(4px); */
  /* border: 1px solid rgba(255, 255, 255, 0.18); */
  border-radius: 0.5rem;
  padding-left: 1rem;
  padding-right: 1rem;
  color: #fafafe;
}
</style>
