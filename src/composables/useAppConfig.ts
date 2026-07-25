import { ref, type Ref } from "vue";
import type { AppConfig } from "../types/models";
import { getConfig } from "../ipc/api";

const config: Ref<AppConfig | null> = ref(null);
const loading = ref(false);
const error = ref<string | null>(null);

export function useAppConfig() {
  async function refresh() {
    loading.value = true;
    error.value = null;
    try {
      config.value = await getConfig();
    } catch (e: any) {
      error.value = e?.message ?? String(e);
    } finally {
      loading.value = false;
    }
  }

  return { config, loading, error, refresh };
}
