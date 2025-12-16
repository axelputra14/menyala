<script setup>
import { ref } from "vue";
import { useKeenSlider } from "keen-slider/vue.es";
import "keen-slider/keen-slider.min.css";
import { useFetchStore } from "../stores/fetch";
import { onMounted } from "vue";

import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";

const fetchStore = useFetchStore();

const emit = defineEmits(["hover", "leave"]);

// const hoveredApp = ref(null);
const hoveredIndex = ref(null);

function iconStyle(index) {
  if (hoveredIndex.value === null) return {};

  const distance = Math.abs(index - hoveredIndex.value);

  let scale = 1;
  let translateY = 0;

  if (distance === 0) {
    scale = 1.5;
    translateY = -14;
  } else if (distance === 1) {
    scale = 1.25;
    translateY = -8;
  } else if (distance === 2) {
    scale = 1.1;
    translateY = -4;
  }

  return {
    transform: `translateY(${translateY}px) scale(${scale})`,
    zIndex: 10 - distance,
  };
}
function onHover(app, index) {
  hoveredIndex.value = index;
  emit("hover", app.name);
}

function onLeave() {
  hoveredIndex.value = null;
  emit("leave");
}

function launchApp(exe) {
  invoke("launch_app", { exe });
}

onMounted(async () => {
  await fetchStore.getApps();
  await fetchStore.refreshApps();
});

const wheelControls = (slider) => {
  let touchTimeout;
  let position = { x: 0 };
  let wheelActive = true;

  const dispatch = (e, name) => {
    position.x -= e.deltaY;
    slider.container.dispatchEvent(
      new CustomEvent(name, { detail: { x: position.x, y: 0 } })
    );
  };

  const eventWheel = (e) => {
    e.preventDefault();
    if (!wheelActive) {
      position = { x: 0 };
      dispatch(e, "ksDragStart");
      wheelActive = true;
    }
    dispatch(e, "ksDrag");
    clearTimeout(touchTimeout);
    touchTimeout = setTimeout(() => {
      wheelActive = false;
      dispatch(e, "ksDragEnd");
    }, 50);
  };

  slider.on("created", () => {
    slider.container.addEventListener("wheel", eventWheel, { passive: false });
  });
};

const [container] = useKeenSlider(
  {
    loop: true,
    mode: "snap",
    rubberband: false,
    slides: { perView: "auto", spacing: 16 },
  },
  [wheelControls]
);
</script>

<template>
  <div class="slider-wrapper">
    <div ref="container" class="keen-slider">
      <div
        v-for="(app, index) in fetchStore.appList"
        :key="app.exe"
        class="keen-slider__slide slide-items"
        @mouseenter="onHover(app, index)"
        @mouseleave="onLeave()"
        @click="launchApp(app.exe)"
      >
        <img
          :src="convertFileSrc(app.icon)"
          v-bind:alt="app.name"
          width="48"
          height="48"
          :style="iconStyle(index)"
        />
      </div>
    </div>
  </div>
</template>

<style>
/* .tooltip-container {

} */

body {
  margin: 0;
  font-family: "Inter", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.slider-wrapper {
  position: relative; /* needed for overlays */
  background: rgba(0, 0, 0, 0.3);
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
  backdrop-filter: blur(1px);
  -webkit-backdrop-filter: blur(4px);
  border: 1px solid rgba(115, 115, 155, 0.4);
  border-radius: 2rem;
  padding: 8px 16px;
  margin-bottom: 1rem;
  margin-top: 3vh;
}

/* gradient overlays on left + right */
.slider-wrapper::before,
.slider-wrapper::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 48px;
  pointer-events: none;
  z-index: 5;
}

.slider-wrapper::before {
  left: 0;
  background: linear-gradient(
    to right,
    rgba(0, 0, 0, 0.3),
    rgba(255, 255, 255, 0.01)
  );
}

.slider-wrapper::after {
  right: 0;
  background: linear-gradient(
    to left,
    rgba(203, 203, 203, 0.3),
    rgba(255, 255, 255, 0.01)
  );
}

[class^="number-slide"],
[class*=" number-slide"] {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: #030303;
  font-weight: 500;
  max-height: 100vh;
}

.keen-slider {
  overflow: visible !important; /* no mask anymore */
}

.keen-slider__slide {
  height: 64px;
  min-width: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.keen-slider__slide:hover {
  cursor: pointer;
}

.slide-items {
  overflow: visible !important;
  position: relative; /* tooltip positions relative to icon */
}

.slide-items svg,
.slide-items img {
  transition: transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
  transform-origin: 50% 100%;
  will-change: transform;
}

.slide-items svg.loading,
.slide-items img.loading {
  animation: 1s loading ease-in infinite;
}

.slider-wrapper {
  animation: 2s ease-out 0s 1 wait, 2s ease-out 2s 1 slideInFromBottom;
}

@keyframes wait {
  from {
    transform: translateY(150px);
  }
  to {
    transform: translateY(150px);
  }
}

@keyframes slideInFromBottom {
  from {
    transform: translateY(150px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
</style>
