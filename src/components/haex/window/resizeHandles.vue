<template>
  <template v-if="!disabled">
    <div
      class="absolute top-0 left-0 size-2 cursor-nw-resize z-10"
      @mousedown.left.stop="emitResizeStart('nw', $event)"
      @touchstart.passive.stop="emitResizeStart('nw', $event)"
    />
    <div
      class="absolute top-0 right-0 size-2 cursor-ne-resize z-10"
      @mousedown.left.stop="emitResizeStart('ne', $event)"
      @touchstart.passive.stop="emitResizeStart('ne', $event)"
    />
    <div
      class="absolute bottom-0 left-0 size-2 cursor-sw-resize z-10"
      @mousedown.left.stop="emitResizeStart('sw', $event)"
      @touchstart.passive.stop="emitResizeStart('sw', $event)"
    />
    <div
      class="absolute bottom-0 right-0 w-2 h-2 cursor-se-resize z-10"
      @mousedown.left.stop="emitResizeStart('se', $event)"
      @touchstart.passive.stop="emitResizeStart('se', $event)"
    />
    <div
      class="absolute top-0 left-2 right-2 h-2 cursor-n-resize z-10"
      @mousedown.left.stop="emitResizeStart('n', $event)"
      @touchstart.passive.stop="emitResizeStart('n', $event)"
    />
    <div
      class="absolute bottom-0 left-2 right-2 h-2 cursor-s-resize z-10"
      @mousedown.left.stop="emitResizeStart('s', $event)"
      @touchstart.passive.stop="emitResizeStart('s', $event)"
    />
    <div
      class="absolute left-0 top-2 bottom-2 w-2 cursor-w-resize z-10"
      @mousedown.left.stop="emitResizeStart('w', $event)"
      @touchstart.passive.stop="emitResizeStart('w', $event)"
    />
    <div
      class="absolute right-0 top-2 bottom-2 w-2 cursor-e-resize z-10"
      @mousedown.left.stop="emitResizeStart('e', $event)"
      @touchstart.passive.stop="emitResizeStart('e', $event)"
    />
  </template>
</template>

<script setup lang="ts">
// Props: Nur Information, ob Handles angezeigt werden sollen
defineProps<{
  disabled?: boolean // True if window is maximized
}>()

// Emits: Signalisiert den Start des Resizing mit Richtung und Event
const emit = defineEmits<{
  resizeStart: [direction: string, event: MouseEvent | TouchEvent]
}>()

// Funktion, um das Event nach oben weiterzuleiten
const emitResizeStart = (direction: string, event: MouseEvent | TouchEvent) => {
  emit('resizeStart', direction, event)
}
</script>
