<script setup lang="ts"></script>

<template>
  <div class="tooltip-container">
    <div class="text-box">
      <p>AstralAir no Shiroki Towa</p>
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
}

.text-box {
  /* background: rgba(255, 255, 255, 0.1); */
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.17);
  /* backdrop-filter: blur(1px); */
  /* -webkit-backdrop-filter: blur(4px); */
  /* border: 1px solid rgba(255, 255, 255, 0.18); */
  border-radius: 1rem;
  padding-left: 8px;
  padding-right: 4px;
}
</style>
