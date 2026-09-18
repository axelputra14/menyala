<script setup>
import { ref, onMounted, nextTick, watch } from "vue";
import { useKeenSlider } from "keen-slider/vue.es";
import "keen-slider/keen-slider.min.css";
import { useFetchStore } from "../stores/fetch";

import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";

const fetchStore = useFetchStore();

// const loading = ref(true);

const emit = defineEmits(["hover", "leave"]);

// const hoveredApp = ref(null);
const hoveredIndex = ref(null);
const pressedIndex = ref(null);

function iconStyle(index) {
  let scale = 1;
  let translateY = 0;
  if (hoveredIndex.value === null) return {};

  const distance = Math.abs(index - hoveredIndex.value);

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

  if (pressedIndex.value === index) {
    translateY -= 6;
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

function launchApp(exe, index) {
  pressedIndex.value = index;

  // visual feedback first
  setTimeout(() => {
    pressedIndex.value = null;
  }, 120);

  // launch slightly after press starts
  setTimeout(() => {
    invoke("launch_app", { exe });
  }, 40);
}
const wheelControls = (slider) => {
  let touchTimeout;
  let position = { x: 0 };
  let wheelActive = true;

  const dispatch = (e, name) => {
    position.x -= e.deltaY;
    slider.container.dispatchEvent(
      new CustomEvent(name, { detail: { x: position.x, y: 0 } }),
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

const [container, slider] = useKeenSlider(
  {
    loop: true,
    mode: "free-snap",
    rubberband: true,
    slides: { perView: 8, spacing: 32 },
  },
  [wheelControls],
);

watch(
  () => fetchStore.appList,
  async () => {
    await nextTick();
    slider.value?.update();
  },
  { deep: true, flush: "post" },
);

onMounted(async () => {
  await fetchStore.getApps();
  await fetchStore.refreshApps();
  slider.value?.update();
});
</script>

<template>
  <div class="slider-wrapper">
    <div ref="container" class="keen-slider">
      <div
        v-for="(app, index) in fetchStore.appList"
        :key="app.id ?? app.exe"
        class="keen-slider__slide slide-items"
        @mouseenter="onHover(app, index)"
        @mouseleave="onLeave()"
        @click="launchApp(app.exe, index)"
      >
        <img
          :src="convertFileSrc(app.icon)"
          v-bind:alt="app.name"
          width="64"
          height="64"
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
  border: 1px solid rgba(115, 115, 155, 0.4);
  border-radius: 1.5rem;
  padding: 8px 16px;
  margin-bottom: 0.25rem;
  margin-top: 3vh;
  /* macOS dock-style: bottom-only shadow via negative spread */
  box-shadow: 0 12px 24px -20px rgba(0, 0, 0, 0.6);
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
  border-radius: 1.5rem;
}

.slider-wrapper::before {
  left: 0;
  background: linear-gradient(
    to right,
    rgba(0, 0, 0, 0.3),
    rgba(255, 255, 255, 0.01)
  );
  border-radius: 1.5rem;
}

.slider-wrapper::after {
  right: 0;
  background: linear-gradient(
    to left,
    rgba(0, 0, 0, 0.3),
    rgba(255, 255, 255, 0.01)
  );
  border-radius: 1.5rem;
}

.keen-slider {
  overflow: visible !important; /* no mask anymore */
}

.keen-slider__slide {
  max-height: 64px;
  max-width: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.keen-slider__slide img {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
  transition: transform 160ms ease-out;
  transform-origin: 50% 100%;
  will-change: transform;
}

.slide-items svg.loading,
.slide-items img.loading {
  animation: 1s loading ease-in infinite;
}
</style>
