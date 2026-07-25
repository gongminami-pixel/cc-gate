import { ref } from "vue";

export interface Toast {
  id: number;
  kind: "ok" | "err" | "info";
  message: string;
}

const toasts = ref<Toast[]>([]);
let nextId = 1;

export function useToast() {
  function add(kind: Toast["kind"], message: string) {
    const t: Toast = { id: nextId++, kind, message };
    toasts.value = [...toasts.value, t];
    setTimeout(() => remove(t.id), 4000);
  }

  function remove(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  return {
    toasts,
    ok: (msg: string) => add("ok", msg),
    err: (msg: string) => add("err", msg),
    info: (msg: string) => add("info", msg),
    remove,
  };
}
