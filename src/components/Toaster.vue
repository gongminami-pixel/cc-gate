<script setup lang="ts">
import { useToast } from "../composables/useToast";
const { toasts, remove } = useToast();
</script>

<template>
  <div class="toaster">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
      <span class="msg">{{ t.message }}</span>
      <button class="close" @click="remove(t.id)">&times;</button>
    </div>
  </div>
</template>
