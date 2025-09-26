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
  position: absolute;
  bottom: 100%; /* place above icon */
  margin-bottom: 6px;

  background: rgba(0, 0, 0, 0.6);
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.17);
  color: white;
  font-size: 14px;
  padding: 4px 1rem;
  border-radius: 8px;

  /* hidden by default */
  opacity: 0;
  transform: translateY(15px);
  transition: opacity 0.2s ease, transform 0.2s ease;
  pointer-events: none;
  z-index: 9999;
}
</style>
