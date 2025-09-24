<script setup>
import { useKeenSlider } from "keen-slider/vue.es";
import "keen-slider/keen-slider.min.css";
import { useFetchStore } from "../stores/fetch";
import Tooltip from "./Tooltip.vue";

const fetchStore = useFetchStore();

onMounted(() => {
  fetchStore.getApps();
});

const wheelControls = (slider) => {
  let touchTimeout;
  let position = { x: 0 };
  let wheelActive = false;

  const dispatch = (e, name) => {
    // Use vertical wheel movement to drag horizontally
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
  <Tooltip />
  <div class="slider-wrapper">
    <div ref="container" class="keen-slider">
      <div
        v-for="app in fetchStore.appList"
        :key="app.exe"
        class="keen-slider__slide slide-items"
        :class="`number-slide-${app}`"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="48"
          height="48"
          viewBox="0 0 32 32"
        >
          <path
            fill="#111111"
            d="M11.807 9.776c.011 0 .005 0 0 0M8.109 7.927c.011 0 .005 0 0 0m22.12 2.854c-.667-1.604-2.021-3.333-3.079-3.885c.865 1.692 1.365 3.396 1.552 4.661l.005.027c-1.739-4.329-4.681-6.073-7.088-9.871c-.12-.192-.24-.385-.36-.588c-.063-.104-.115-.208-.172-.319a2.8 2.8 0 0 1-.224-.609c0-.02-.015-.036-.036-.041h-.031l-.005.005c-.005 0-.011.005-.011.005s0-.005.005-.011c-3.417 2-4.828 5.505-5.193 7.729a8.3 8.3 0 0 0-3.041.776a.396.396 0 0 0-.197.489a.387.387 0 0 0 .525.224a7.4 7.4 0 0 1 2.651-.687l.089-.011c.125-.005.255-.011.38-.011a7.7 7.7 0 0 1 2.203.307l.125.037c.12.036.235.077.355.12c.083.031.172.063.255.099c.068.025.136.057.203.083q.157.073.313.152l.14.067q.155.08.303.167q.094.054.187.115a7.8 7.8 0 0 1 2.683 2.776c-.817-.572-2.287-1.145-3.697-.895c5.52 2.76 4.036 12.265-3.615 11.905a6.6 6.6 0 0 1-2.448-.568l-.26-.124c-1.876-.969-3.423-2.803-3.615-5.027c0 0 .708-2.64 5.072-2.64c.475 0 1.824-1.319 1.849-1.699c-.011-.125-2.683-1.187-3.724-2.213c-.557-.547-.817-.812-1.052-1.011a4 4 0 0 0-.401-.301a7.1 7.1 0 0 1-.041-3.751c-1.579.719-2.803 1.855-3.693 2.855h-.009c-.609-.771-.563-3.313-.532-3.844c-.005-.036-.453.229-.511.271c-.536.385-1.041.813-1.5 1.287a13.5 13.5 0 0 0-1.437 1.719a13 13 0 0 0-2.057 4.645a18 18 0 0 0-.249 1.417a8 8 0 0 0-.052.359a10 10 0 0 0-.089.881L.7 15.9c-.009.172-.02.339-.031.511v.077c0 8.48 6.875 15.355 15.355 15.355c7.593 0 13.9-5.516 15.135-12.756c.027-.197.047-.395.068-.593c.307-2.631-.031-5.401-.995-7.713z"
          />
        </svg>
      </div>
    </div>
  </div>
</template>

<style>
body {
  margin: 0;
  font-family: "Inter", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.slider-wrapper {
  background: rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
  backdrop-filter: blur(1px);
  -webkit-backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 1rem;
  padding-left: 16px;
  padding-right: 16px;
  padding-top: 8px;
  padding-bottom: 8px;
  margin-bottom: 1rem;
  overflow: hidden;
}

[class^="number-slide"],
[class*=" number-slide"] {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: #fff;
  font-weight: 500;
  max-height: 100vh;
}

.keen-slider {
  -webkit-mask-image: linear-gradient(
    to right,
    transparent,
    black 10%,
    black 90%,
    transparent
  );
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: 100% 100%;

  mask-image: linear-gradient(
    to right,
    transparent,
    black 11%,
    black 89%,
    transparent
  );
  mask-repeat: no-repeat;
  mask-size: 100% 100%;
}

.keen-slider__slide {
  height: 64px;
  min-width: 48px;
}

.keen-slider__slide:hover {
  cursor: pointer;
}

.slide-items {
  overflow: visible;
}

.slide-items svg,
.slide-items img {
  transition: 250ms all ease-in-out;
}

.slide-items:hover svg,
.slide-items:hover img {
  transform: translateY(-5px) scale(1.05);
}

.slide-items svg.loading,
.slide-items img.loading {
  animation: 1s loading ease-in infinite;
}
</style>
