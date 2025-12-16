<script setup lang="ts">
defineProps<{
  text: string;
  visible: boolean;
}>();
</script>

<template>
  <div class="tooltip-container" :class="{ visible }">
    {{ text }}
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=Open+Sans:ital,wght@0,300..800;1,300..800&display=swap");

.tooltip-container {
  font-family: "Open Sans", sans-serif;
  font-weight: 500;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: absolute;
  bottom: 50%; /* place above icon */
  position: relative;
  flex: auto;
  justify-content: center;
  align-items: center;

  background: rgba(0, 0, 0, 0.6);
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.17);
  color: white;
  padding: 4px 1rem;
  border-radius: 8px;

  width: max-content; /* shrink-to-fit */
  white-space: normal; /* allow wrapping */
  word-break: break-word; /* handle long names */
  overflow: hidden;
  width: fit-content;
  max-width: 400px;

  display: -webkit-box;
  -webkit-box-orient: vertical;
  line-clamp: 3;
  -webkit-line-clamp: 3;

  padding: 4px 1rem;
  border-radius: 8px;

  /* hidden by default */
  opacity: 0;
  transform: translateY(15px);
  transition: opacity 0.2s ease, transform 0.2s ease;
  pointer-events: none;
  z-index: 9999;
}
.tooltip-container.visible {
  opacity: 1;
  transform: translateY(0);
}
</style>
