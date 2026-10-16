// dashboard/lib/types.ts

export type ContainerStatus = "Running" | "Stopped" | "Crashed"

export type Container = {
    id: string
    name: string
    image: string
    args: string[]
    status: ContainerStatus
    pid: number | null
}

export type Metrics = {
    cpu_usage: number
    memory_used: number
    memory_total: number
    uptime_secs: number
}

export type ContainerMetrics = {
    container_id: string
    pid: number
    cpu_percent: number
    memory_rss_kb: number
}

export type StorageFile = string