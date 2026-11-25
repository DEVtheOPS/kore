import { createContext } from "svelte";
import type { Cluster } from "$lib/stores/clusters.svelte";

export const [getClusterMetadata, setClusterMetadata] = createContext<() => Cluster | null>();
